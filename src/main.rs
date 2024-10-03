use tiny_http::{Server, Request, Response, Method, Header};
use std::env::args;
use local_ip_address::local_ip;
use std::net::{IpAddr, Ipv4Addr};
use std::fs::{File, read_to_string, self};
use std::io::{Write, Cursor};
use std::thread;
use std::collections::{HashSet, HashMap};
use std::str::FromStr;
use tera::{Tera, Context};
use ws::{Handler, Factory, Sender, Handshake, Message, CloseCode, listen, WebSocket};
use std::sync::mpsc;
use spmc;
use toml::{Table, de};
use serde::Deserialize;

const PAGES_DIR: &str = "/home/sasha/Projects/bang-web/src/pages";
const CONFIG_FILE: &str = "/home/sasha/Projects/bang-web/src/config.toml";
const DF_MIN_PLAYERS: u8 = 4;
const DF_MAX_PLAYERS: u8 = 7;
const DF_CARDS: [(Cards, u8, u8, u8, u8); 22] = [
    (Cards::Bang, 1, 13, 8, 3),
    (Cards::Miss, 1, 13, 8, 3),
    (Cards::Indians, 1, 13, 8, 3),
    (Cards::Jail, 1, 13, 8, 3),
    (Cards::Barrel, 1, 13, 8, 3),
    (Cards::Mustang, 1, 13, 8, 3),
    (Cards::Beer, 1, 13, 8, 3),
    (Cards::CatBalou, 1, 13, 8, 3),
    (Cards::Duel, 1, 13, 8, 3),
    (Cards::Gatling, 1, 13, 8, 3),
    (Cards::Store, 1, 13, 8, 3),
    (Cards::Panic, 1, 13, 8, 3),
    (Cards::Saloon, 1, 13, 8, 3),
    (Cards::Stagecoach, 1, 13, 8, 3),
    (Cards::WellsFargo, 1, 13, 8, 3),
    (Cards::Dynamite, 1, 13, 8, 3),
    (Cards::Schofield, 1, 13, 8, 3),
    (Cards::Volcanic, 1, 13, 8, 3),
    (Cards::Remington, 1, 13, 8, 3),
    (Cards::Carabine, 1, 13, 8, 3),
    (Cards::Winchester, 1, 13, 8, 3),
    (Cards::Scope, 1, 13, 8, 3),
];
const DF_CHARACTERS: [Characters; 16] = [
    Characters::CalamityJanet,
    Characters::SlabTheKiller,
    Characters::WillyTheKid,
    Characters::PaulRegret,
    Characters::Jourdonnais,
    Characters::RoseDoolan,
    Characters::BlackJack,
    Characters::PedroRamirez,
    Characters::BartCassidy,
    Characters::ElGringo,
    Characters::JesseJones,
    Characters::KitCarlson,
    Characters::LuckyDuke,
    Characters::SidKetchum,
    Characters::SuzyLafayette,
    Characters::VultureSam
];


fn main() -> Result<(), ()> {
    //TODO: WebSockets
    //TODO: Implement all of the features in the config struct
    //TODO: Lobbies
    let args: Vec<_> = args().collect();
    let (address, port) = match set_address(args) {
        Ok(content) => content,
        Err(_) => {
          Err(())
        }?
    };
    //let url = format!("{address}:{port}");
    let table: Table = parse_config().expect("TOML - Could not parse\n");
    let config = set_config(table.clone());
    start_server(address.to_string(), port, config)?;
    Err(())
}
#[derive(Deserialize, Debug)]
struct Config {
    general: ConfigGeneral,
    gameplay: ConfigGameplay,
    stats: ConfigStats,
    extra: ConfigExtra,
}
#[derive(Deserialize, Debug)]
struct ConfigGeneral {
    min_players: Option<u8>,
    max_players: Option<u8>,
    extras: Option<bool>,
}
#[derive(Deserialize, Debug)]
struct ConfigGameplay {
    weapons: Option<HashSet<String>>,
    characters: Option<Vec<String>>,
    cards: Option<HashSet<(String, u8, u8, u8, u8)>>,
}
#[derive(Deserialize, Debug)]
struct ConfigStats {
    max_health: Option<u8>,
    sheriff_max_health: Option<u8>,
    lower_max_health_lowering_amount: Option<u8>,
    weapon_ranges: Option<HashSet<(String, u8)>>,
    mustang_extra_distance: Option<u8>,
    scope_extra_range: Option<u8>,
}
#[derive(Deserialize, Debug)]
struct ConfigExtra {
    targets: Option<HashSet<String>>,
    beer_revive: Option<bool>,
    stack_mustang_and_scope: Option<bool>,
}
impl FromStr for Weapons {
    type Err = ();
    fn from_str(input: &str) -> Result<Weapons, Self::Err> {
        match input {
            "Schofield"  => Ok(Weapons::Schofield),
            "Remington"  => Ok(Weapons::Remington),
            "Carabine"  => Ok(Weapons::Carabine),
            "Winchester" => Ok(Weapons::Winchester),
            "Volcanic" => Ok(Weapons::Volcanic),
            _ => Err(()),
        }
    }
}
impl FromStr for Characters {
    type Err = ();
    fn from_str(input: &str) -> Result<Characters, Self::Err> {
        match input {
            "Calamity Janet"  => Ok(Characters::CalamityJanet),
            "Slab the Killer"  => Ok(Characters::SlabTheKiller),
            "Willy the Kid"  => Ok(Characters::WillyTheKid),
            "Paul Regret"  => Ok(Characters::PaulRegret),
            "Jourdonnais" => Ok(Characters::Jourdonnais),
            "Rose Doolan" => Ok(Characters::RoseDoolan),
            "Black Jack" => Ok(Characters::BlackJack),
            _ => Err(()),
        }
    }
}
impl FromStr for Cards {
    type Err = ();
    fn from_str(input: &str) -> Result<Cards, Self::Err> {
        match input {
            "Bang"  => Ok(Cards::Bang),
            "Miss"  => Ok(Cards::Miss),
            "Jail"  => Ok(Cards::Jail),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
enum Weapons {
    Colt45,
    Volcanic,
    Schofield,
    Remington,
    Carabine,
    Winchester
}
#[derive(Deserialize, Debug, Clone)]
enum Characters {
    CalamityJanet,
    SlabTheKiller,
    WillyTheKid,
    PaulRegret,
    Jourdonnais,
    RoseDoolan,
    BlackJack,
    PedroRamirez,
    BartCassidy,
    ElGringo,
    JesseJones,
    KitCarlson,
    LuckyDuke,
    SidKetchum,
    SuzyLafayette,
    VultureSam
}
#[derive(Deserialize, Debug, Clone)]
enum Roles {
    Outlaw,
    Sheriff,
    Renegade,
    Deputy
}
#[derive(Deserialize, Debug, Clone)]
enum Cards {
    Bang,
    Miss,
    Indians,
    Jail,
    Barrel,
    Mustang,
    Beer,
    CatBalou,
    Duel,
    Gatling,
    Store,
    Panic,
    Saloon,
    Stagecoach,
    WellsFargo,
    Dynamite,
    Schofield,
    Volcanic,
    Remington,
    Carabine,
    Winchester,
    Scope
}
#[derive(Deserialize, Debug, Clone)]
enum Suits {
    Heart,
    Diamond,
    Spade,
    Club
}
#[derive(Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
enum Attributes {
    LowerMaxHP,
    Barrel,
    Targeted,
    Dynamite,
    Mustang,
    Scope,
    Jailed,
    BangSpam,
    ExtraDistance,
    ExtraRange,
}

#[derive(Debug, Clone)]
struct Player {
    name: String,
    health: u8,
    weapon: Weapons,
    character: Characters,
    role: Roles,
    upper_cards: Vec<(Cards, Suits)>,
    lower_cards: Vec<Cards>,
    attributes: HashSet<Attributes>,
    attributes_num: HashMap<Attributes, u8>,
}
/*enum Events {
    DeckPull,
    CardPlay(u8),
    LowerDeckAdd(u8),
    LowerDeckRemove(u8),
    ChangeWeapon(u8),
}*/

struct EventHandler {
    ws: Sender,
    game_updater: (mpsc::Sender<String>, spmc::Receiver<String>),
    player_count: u8,
    max_player_count: u8,
}
impl Handler for EventHandler {
    fn on_open(&mut self, shake: Handshake) -> Result<(), ws::Error> {
        println!("Connection made, ID: {}", self.player_count);
        self.ws.send("hi");
        if self.player_count > self.max_player_count {
            println!("Max player count has been reached, disconnecting WebSocket client");
            self.ws.close(CloseCode::Invalid);
        }
        //TODO: Close WebSocket if max player count has been reached
        //TODO: If the game has started, close unregistered clients (those who didn't connect
        //during waiting)
        //TODO: Make a hash of an id, then have it be stored in Local Storage, if a player is new,
        //generate one with a salt
        Ok(())
    } 
    fn on_message(&mut self, msg: Message) -> Result<(), ws::Error> {
        println!("Message received: {msg}");
        //TODO: Change game state and update other clients (partial and full state updates)
        self.ws.broadcast(msg.clone());
        self.game_updater.0.send(msg.into_text()?);
        let pog = self.game_updater.1.recv().expect("MPSC\n");
        println!("{}", pog);
        self.ws.send("abcd");
        Ok(())
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Connection closed: CODE {code:?} - {reason}");
    }
}
/*impl EventHandler {
    fn change_state(&self, msg: Message) -> Result<(), ()> {
        self.game_updater.0.send("pog".to_string());
        Ok(())
    }
}*/
struct HandlerFactory {
    player_count: u8,
    max_player_count: u8,
    game_updater: (mpsc::Sender<String>, spmc::Receiver<String>),
}
impl Factory for HandlerFactory {
    type Handler = EventHandler;
    fn connection_made(&mut self, ws: Sender) -> EventHandler {
        let handler = EventHandler {
            ws: ws,
            player_count: self.player_count,
            max_player_count: self.max_player_count,
            game_updater: self.game_updater.clone(),
        };
        self.player_count += 1;
        println!("{}", self.player_count);
        handler
    }
    fn connection_lost(&mut self, _: Self::Handler) {
        self.player_count -= 1;
        println!("{}", self.player_count);
    }
}
/*macro_rules! remove_attr {
    ($set:expr, $variant:pat) => {
        {
            let attr = $set
                .clone()
                .into_iter()
                .filter(|k| matches!(k, $variant))
                .last()
                .unwrap();
            let boolean = $set.remove(&attr);
            ($set, attr)
        }
    }
}*/
fn start_server(url: String, port: u16, config: Result<Config, ()>) -> Result<(), ()> {
    //println!("{:?}", config.unwrap());
    //match config {}
    let server = Server::http(&format!("{}:{}", &url, &port)).expect("TinyHTTP - Could not start server");
    println!("Server running at {}:{}", &url, &port);
    let (txWS, rxGame) = mpsc::channel::<String>();
    let (mut txGame, rxWS) = spmc::channel::<String>();
    let ws = WebSocket::new(HandlerFactory{max_player_count: 4, player_count: 1, game_updater: (txWS, rxWS)}).expect("WebSocket\n");
    thread::scope(|s| {
        s.spawn(move || { // HTTP Server Thread
            loop {
                let mut request = server.recv().expect("TinyHTTP - Could not receive request");
                println!("RQ: {} {}", request.method(), request.url());
                let mut tera = Tera::new(&format!("{PAGES_DIR}/*.html")).expect("Tera\n");
                tera.add_template_files(vec![
                    (&file("start.html"), Some("start")),   
                    (&file("index.html"), Some("base")),   
                    //(&file("htmx"), Some("htmx"))
                ]);
                match (request.url(), request.method()) {
                    ("/" | "/index.html", Method::Get) => {
                        serve_file("index.html", request);
                    }
                    ("/htmx", Method::Get) => {
                        serve_file("htmx.js", request);
                    }
                    ("/ws.js", Method::Get) => {
                        serve_file("ws.js", request);
                    }
                    /*("/ws", Method::Get) => {
                        let mut socket = request.upgrade("ws", Response::from_string("pog"));
                        let mut jog: [u8; 0] = [];
                        let pog = socket.read(&mut jog);
                        println!("{:?}", jog);
                    }*/
                    ("/start", Method::Post) => {
                        let pairs = vec![
                            ("pog", "helo"), 
                            ("submit", "Time to submit")
                        ];
                        let body = get_request_body(&mut request);
                        println!("pog: {}", body);
                        let response = template("start.html", tera, "start", pairs);
                        request.respond(response);
                    }
                    ("/revolver" | "/favicon.ico", Method::Get) => {
                        serve_file("revolver.png", request);
                    }
                    ("/background", Method::Get) => {
                        serve_file("background.png", request);
                    }
                    ("/enterGame", Method::Post) => {
                        let body = get_request_body(&mut request);
                        println!("{}", body);
                        serve_file("waiting.html", request);
                    }
                    /*(request_url, Method::Get) => {
                        let rq_url = request_url.to_string();
                        serve_file(&rq_url, request);
                    }*/
                    _ => {
                        println!("Invalid request");
                        request.respond(Response::from_string("404"));
                    }
                }
            }
        });
        s.spawn(move || { // WebSocket Thread
            ws.listen(&format!("{}:{}", &url, &port + 1)).expect("WebSocket Listener\n");
        });
        s.spawn(move || { // Game Thread
            let mut player1 = Player {name: "pogger".to_string(), role: Roles::Deputy, health: 1u8, character: Characters::SlabTheKiller, weapon: Weapons::Schofield, upper_cards: vec![(Cards::Bang, Suits::Club)], lower_cards: vec![Cards::Mustang], attributes: HashSet::new(), attributes_num: HashMap::new()};
            player1.attributes_num.insert(Attributes::Mustang, 3);
            player1.attributes_num.insert(Attributes::Mustang, 2);
            //player1.attributes.insert(Attributes::Mustang(3));
            //let (set, removed) = remove_attr!(player1.attributes, Attributes::Mustang(u8));
            //player1.attributes = set;
            /*if let Attributes::Mustang(num) = removed {
                player1.attributes.insert(Attributes::Mustang(num + 1));
            };*/
            println!("{:?}", player1.attributes_num);
            loop {
                let msg = rxGame.recv().unwrap_or_else(|err| {
                    eprintln!("ERROR: Game Server - {err}");
                    "".to_string()
                });
                let split: Vec<_> = msg.split(":").collect();
                if split.len() != 2 {
                    txGame.send("Invalid".to_string());
                    continue;
                }
                let command = split[0];
                let content = split[1];
                let return_content = match command {
                    "playCard" => play_card(),
                    "pullCard" => pull_card(),
                    "lowerDeckAdd" => lower_deck_add(),
                    "lowerDeckRemove" => lower_deck_remove(),
                    "changeWeapon" => change_weapon(),
                    _ => println!("other")
                };
                println!("Game: {:?}", content);
                //thread::sleep(Duration::from_secs(3));
                println!("Game: {:?}", msg);
                txGame.send("Game updated".to_string());
            }
        });
    });
    Ok(())
}
fn set_address(args: Vec<String>) -> Result<(String, u16), String> {
    let mut port: u16 = Default::default();
    let mut address: String = Default::default();
    match args.len() {
        2 => {
            let ip_addr = match local_ip() {
                Ok(addr) => addr,
                Err(err) => Err(err.to_string())?
            };
            address = ip_addr.to_string();
        }
        3 => {
            address = args[2].clone();
        }
        _ => {
            return Err("Wrong amount of args".to_string());
        }
    }
    match args[1].parse::<u16>() {
        Ok(num) => port = num,
        Err(_) => {
            Err("Port number is not a u16")
        }?
    };
    Ok((address.to_string(), port))
}
fn file(file_name: &str) -> String {
    format!("{PAGES_DIR}/{file_name}")
}
fn read_file(file_name: &str) -> String {
    let path = file(file_name);
    let content = read_to_string(path).unwrap_or_else(|err| {
        eprintln!("ERROR  File - {err}");
        "".to_string()
    });
    content
}
fn override_file(file_name: &str, content: String) {
    fs::write(&file(file_name), content.as_bytes());
}
fn append_to_file(file_name: &str, content: String) {
    let mut file = File::options()
        .append(true)
        .create(true)
        .open(&file(file_name))
        .expect("File Handler (appending)\n");
    file.write_all(content.as_bytes());
}
fn serve_file(file_name: &str, request: Request) {
    let path = file(file_name);
    let file = File::open(&path).unwrap_or_else(|err| {
        eprintln!("ERROR: File - {err}");
        File::create(&path).expect("File Handler (serving)\n")
    });
    request.respond(Response::from_file(file));
}
fn render_html(file_name: &str, mut tera: Tera, page: &str, substitution: Vec<(&str, &str)>) -> String {
    let file_content = read_file(file_name);
    tera.build_inheritance_chains();
    let mut context = Context::new();
    for pair in substitution {
        let (key, value) = pair;
        context.insert(key, value);
    }
    let rendered = tera.render(page, &context).unwrap_or_else(|err| {
        eprintln!("ERROR: Tera - {err}");
        file_content
    });
    rendered
}
fn template(file_name: &str, tera: Tera, page: &str, pairs: Vec<(&str, &str)>) -> Response<Cursor<Vec<u8>>> {
    let rendered_html = render_html(file_name, tera, page, pairs);
    let mut response = Response::from_string(rendered_html);
    let header = Header::from_bytes(&*b"Content-Type", &*b"text/html").expect("Header\n");
    response.add_header(header);
    response
}
fn get_request_body(request: &mut Request) -> String {
    let mut buffer: String = Default::default();
    let _ = request.as_reader().read_to_string(&mut buffer);
    buffer
}
fn parse_config() -> Result<Table, de::Error> {
    let content = read_to_string(CONFIG_FILE).unwrap_or_else(|err| {
        eprintln!("ERROR: File - {err}");
        "".to_string()
    });
    let value = content.parse::<Table>();
    value
}
fn set_config(table: Table) -> Result<Config, ()> {
    let content = read_to_string(CONFIG_FILE).map_err(|err| {
        eprintln!("ERROR: File - {err}");
    })?;
    let config = toml::from_str::<Config>(&content).map_err(|err| {
        eprintln!("ERROR: TOML - {err}");
        ()
    })?;
    Ok(config)
}
fn pull_card() {
    println!("pogge");
}
fn play_card() {}
fn lower_deck_add() {}
fn lower_deck_remove() {}
fn change_weapon() {}
