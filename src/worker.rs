use crossbeam_channel::{bounded, select, Receiver, Sender};
use failure::{bail, Error};
use karaoke::{
    channel::{
        LiveCommand, PlayerCommand, WebsocketCommand, WorkerCommand, LIVE_CHANNEL, PLAYER_CHANNEL,
        WORKER_CHANNEL,
    },
    collection::Kfile,
    queue::PLAY_QUEUE,
    CONFIG,
};
use multiqueue::{BroadcastReceiver, BroadcastSender};
use std::{
    sync::{Arc, Mutex},
    thread, time,
};
use websocket::{sync::Server, OwnedMessage};

pub fn run() {
    thread::spawn(move || {
        if CONFIG.use_web_player {
            web_worker();
        } else {
            native_worker();
        }
    });
}

fn native_worker() {
    let worker = NativeWorker::new();
    loop {
        select! {
            recv(worker.worker_receiver) -> cmd => worker.process_cmd(cmd.unwrap()),
            default() => {},
        }
        thread::sleep(time::Duration::from_millis(50));
    }
}

fn web_worker() {
    let (send, recv) = multiqueue::broadcast_queue::<LiveCommand>(5);
    let mut worker = WebWorker::new(send);

    // Start websocket server, will receive commands on Player Receiver
    thread::spawn(move || loop {
        if let Err(e) = start_server(recv.clone()) {
            println!("{:?}", e);
        };
    });

    loop {
        select! {
            recv(worker.worker_receiver) -> cmd => worker.process_cmd(cmd.unwrap()),
            default() => {},
        }
        thread::sleep(time::Duration::from_millis(50));
    }
}

#[derive(Debug)]
struct NativeWorker {
    worker_receiver: Receiver<WorkerCommand>,
    player_sender: Sender<PlayerCommand>,
    live_sender: Sender<LiveCommand>,
    queue: Arc<Mutex<Vec<Kfile>>>,
}

impl NativeWorker {
    fn new() -> Self {
        let worker_receiver = WORKER_CHANNEL.1.clone();
        let player_sender = PLAYER_CHANNEL.0.clone();
        let live_sender = LIVE_CHANNEL.0.clone();
        let queue = PLAY_QUEUE.clone();
        NativeWorker {
            worker_receiver,
            player_sender,
            live_sender,
            queue,
        }
    }

    fn process_cmd(&self, cmd: WorkerCommand) {
        match cmd {
            WorkerCommand::Stop => self.stop(),
            WorkerCommand::Next => self.next(),
            WorkerCommand::PlayNow { kfile } => self.play_now(kfile),
            WorkerCommand::ClearQueue => self.clear_queue(),
            WorkerCommand::AddQueue { kfile } => self.add_queue(kfile),
        }
    }

    fn stop(&self) {
        self.clear_queue();

        if self.live_sender.is_empty() {
            self.live_sender.send(LiveCommand::Stop).unwrap();
        }
    }

    fn next(&self) {
        let queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            drop(queue);
            return;
        }
        drop(queue);
        self.live_sender.send(LiveCommand::Stop).unwrap();
    }

    fn play_now(&self, kfile: Kfile) {
        self.live_sender.send(LiveCommand::Stop).unwrap();
        self.player_sender
            .send(PlayerCommand::Play { kfile })
            .unwrap();
    }

    fn clear_queue(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
        drop(queue);
    }

    fn add_queue(&self, kfile: Kfile) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(kfile);
        drop(queue);
    }
}

struct WebWorker {
    worker_receiver: Receiver<WorkerCommand>,
    live_sender: BroadcastSender<LiveCommand>,
    queue: Arc<Mutex<Vec<Kfile>>>,
}

impl WebWorker {
    fn new(live_sender: BroadcastSender<LiveCommand>) -> Self {
        let worker_receiver = WORKER_CHANNEL.1.clone();
        let queue = PLAY_QUEUE.clone();

        WebWorker {
            worker_receiver,
            live_sender,
            queue,
        }
    }

    fn process_cmd(&mut self, cmd: WorkerCommand) {
        match cmd {
            WorkerCommand::Stop => self.stop(),
            WorkerCommand::Next => self.next(),
            WorkerCommand::PlayNow { kfile } => self.play_now(kfile),
            WorkerCommand::ClearQueue => self.clear_queue(),
            WorkerCommand::AddQueue { kfile } => self.add_queue(kfile),
        }
    }

    fn stop(&mut self) {
        self.clear_queue();

        if let Err(e) = self.live_sender.try_send(LiveCommand::Stop) {
            println!("Live send error: {:?}", e);
        };
    }

    fn next(&mut self) {
        let mut queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            drop(queue);
            return;
        }
        queue.remove(0);
        drop(queue);
        if let Err(e) = self.live_sender.try_send(LiveCommand::Stop) {
            println!("Live send error: {:?}", e);
        };
    }

    fn play_now(&mut self, kfile: Kfile) {
        let mut queue = self.queue.lock().unwrap();
        if !queue.is_empty() {
            queue.remove(0);
        }
        queue.insert(0, kfile);
        drop(queue);
        if let Err(e) = self.live_sender.try_send(LiveCommand::Stop) {
            println!("Live send error: {:?}", e);
        };
    }

    fn clear_queue(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
        drop(queue);
    }

    fn add_queue(&self, kfile: Kfile) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(kfile);
        drop(queue);
    }
}

fn start_server(receiver: BroadcastReceiver<LiveCommand>) -> Result<(), Error> {
    let mut server = Server::bind("0.0.0.0:9090")?;
    server.set_nonblocking(true)?;

    loop {
        if let Ok(request) = server.accept() {
            let live_receiver = receiver.add_stream();
            thread::spawn(|| {
                if !request.protocols().contains(&"rust-websocket".to_string()) {
                    request.reject().map_err(|(_, e)| e)?;
                    println!("Connection attempted, incorrect protocol");
                    bail!("Incorrect protocol");
                }

                let mut client = request.use_protocol("rust-websocket").accept().unwrap();

                let ip = client.peer_addr().unwrap();

                println!("Connection from {}", ip);

                let message = OwnedMessage::Text("Hello".to_string());
                client.send_message(&message).unwrap();

                let (mut receiver, mut sender) = client.split().unwrap();

                let (command_sender, command_receiver) = bounded(1);

                thread::spawn(move || {
                    let mut now = time::Instant::now();
                    loop {
                        if let Ok(cmd) = live_receiver.try_recv() {
                            match cmd {
                                LiveCommand::Stop => {
                                    let message = OwnedMessage::Text(String::from("Stop"));
                                    if let Err(e) = sender.send_message(&message) {
                                        println!("Websocket erorr: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        select! {
                            recv(command_receiver) -> cmd => {
                                let cmd = cmd.unwrap();
                                println!("Received Command: {:?}", cmd);
                                match cmd {
                                    WebsocketCommand::Close => {
                                        break;
                                    }
                                    WebsocketCommand::Ping { data } => {
                                        let message = websocket::message::Message::pong(data);
                                        if let Err(e) = sender.send_message(&message) {
                                            println!("Websocket erorr: {:?}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                            default() => {},
                        }

                        if now.elapsed().as_secs() >= 20 {
                            now = time::Instant::now();
                            let message = OwnedMessage::Ping(String::from("Ping").into_bytes());
                            if let Err(e) = sender.send_message(&message) {
                                println!("Websocket erorr: {:?}", e);
                                break;
                            }
                        }

                        thread::sleep(time::Duration::from_secs(1));
                    }

                    println!("Sender thread disconnected for: {}", ip);
                });

                for message in receiver.incoming_messages() {
                    let message = message.unwrap();

                    match message {
                        OwnedMessage::Close(_) => {
                            command_sender.send(WebsocketCommand::Close).unwrap();
                            break;
                        }
                        OwnedMessage::Ping(data) => {
                            command_sender
                                .send(WebsocketCommand::Ping { data })
                                .unwrap();
                        }
                        OwnedMessage::Text(text) => {
                            println!("{} sent: {}", ip, text);
                        }
                        OwnedMessage::Pong(_) => {
                            println!("Pong received from: {}", ip);
                        }
                        _ => {}
                    }
                }
                println!("Receiver thread disconnected for: {}", ip);
                Ok(())
            });
        };

        let _ = receiver.try_recv();
        thread::sleep(time::Duration::from_millis(100));
    }
}
