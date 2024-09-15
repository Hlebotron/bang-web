use tiny_http::{Server, Request, Response, Method, Header};
use std::env::args;
use local_ip_address::local_ip;
use std::net::{IpAddr, Ipv4Addr};
use std::fs::{File, read_to_string, self};
use std::io::Write;
use std::thread;
use std::collections::HashSet;
use std::str::FromStr;
use tera::{Tera, Context};
//use url::{Url, ParseError};
use ws::{Handler, Factory, Sender, Handshake, Message, CloseCode, listen};
use std::sync::mpsc;
use toml::{Table, de};
use serde::Deserialize;

const PAGES_DIR: &str = "/home/sasha/Rust/bang_web/src/pages";
const CONFIG_FILE: &str = "/home/sasha/Rust/bang_web/src/config.toml";
const MIN_PLAYERS: u8 = 4;
const MAX_PLAYERS: u8 = 7;
const DEFAULT_CARDS: [(Cards, u8, u8, u8, u8); 6] = [
    (Cards::Bang, 1, 13, 8, 3),
    (Cards::Miss, 1, 13, 8, 3),
    (Cards::Jail, 1, 13, 8, 3),
    (Cards::Indians, 1, 13, 8, 3),
    (Cards::Barrel, 1, 13, 8, 3),
    (Cards::Mustang, 1, 13, 8, 3),
];
const DEFAULT_CHARACTERS: [Characters; 7] = [
    Characters::CalamityJanet,
    Characters::WillyTheKid,
    Characters::SlabTheKiller,
    Characters::RoseDoolan,
    Characters::BlackJack,
    Characters::PaulRegret,
    Characters::Jourdonnais,
];

#[derive(Deserialize, Debug)]
struct Config {
    general: ConfigGeneral,
    gameplay: ConfigGameplay,
    extensions: ConfigExtensions,
    stats: ConfigStats,
}
#[derive(Deserialize, Debug)]
struct ConfigGeneral {
    min_players: Option<u8>,
    max_players: Option<u8>,
    extensions: Option<bool>,
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
}
#[derive(Deserialize, Debug)]
struct ConfigExtensions {
    targets: Option<HashSet<String>>,
    beer_revives: Option<bool>,
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

#[derive(Deserialize, Debug)]
enum Weapons {
    Colt45,
    Volcanic,
    Schofield,
    Remington,
    Carabine,
    Winchester
}
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
enum Roles {
    Outlaw,
    Sheriff,
    Renegade,
    Deputy
}
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug, Hash, Eq, PartialEq)]
enum Attributes {
    LowerMaxHP,
    Barrel,
    Targeted,
    Dynamite,
    ExtraDistance,
    ExtraRange,
    Jailed,
    BangSpam,
}

#[derive(Debug)]
struct Player {
    name: String,
    health: i8,
    weapon: Weapons,
    character: Characters,
    role: Roles,
    cards: Vec<Cards>,
    attributes: HashSet<Attributes>,
}

struct EventHandler {
    ws: Sender,
    id: u16,
}
impl Handler for EventHandler {
    fn on_open(&mut self, shake: Handshake) -> Result<(), ws::Error> {
        println!("Connection has been made, ID: {}", self.id);
        self.ws.send("hello");
        Ok(())
    } 
    fn on_message(&mut self, msg: Message) -> Result<(), ws::Error> {
        println!("Message received: {msg}");
        self.ws.send("response:pog");
        Ok(())
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Connection closed: CODE {code:?} - {reason}");
    }
}
struct HandlerFactory {
    id: u16,
}
impl Factory for HandlerFactory {
    type Handler = EventHandler;
    fn connection_made(&mut self, ws: Sender) -> EventHandler {
        let handler = EventHandler {
            ws: ws,
            id: self.id
        };
        self.id += 1;
        handler
    }
}

fn main() -> Result<(), ()> {
    //TODO: WebSockets
    //TODO: Implement all of the features in the config struct
    let args: Vec<_> = args().collect();
    let (address, port) = match set_address(args) {
        Ok(content) => content,
        Err(_) => {
          Err(())
        }?
    };
    let url = format!("{address}:{port}");
    let table: Table = parse_config().expect("TOML - Could not parse\n");
    let config = set_config(table.clone());
    //let general = &table["general"];
    start_server(url, config)?;
    Err(())
}
fn start_server(url: String, config: Config) -> Result<(), ()> {
    let server = Server::http(&url).expect("TinyHTTP - Could not start server");
    println!("Server running at {}", &url);
    thread::scope(|s| {
        s.spawn(move || {
            loop {
                let mut request = server.recv().expect("TinyHTTP - Could not receive request");
                println!("RQ: {} {}", request.method(), request.url());
                let mut tera = Tera::new(&format!("{PAGES_DIR}/*.html")).unwrap();
                tera.add_template_files(vec![
                    (&file("index.html"), Some("html")),   
                    (&file("htmx"), Some("htmx"))
                ]);
                match (request.url(), request.method()) {
                    ("/" | "/index.html", Method::Get) => {
                        let pairs = vec![
                            ("pog", "helo"), 
                            ("submit", "Time to submit")
                        ];
                        let rendered_html = render_html("index.html", tera, pairs);
                        let mut response = Response::from_string(rendered_html);
                        let header = Header::from_bytes(&*b"Content-Type", &*b"text/html").unwrap();
                        response.add_header(header);
                        request.respond(response);
                    }
                    ("/htmx", Method::Get) => {
                        serve_file("htmx", request);
                    }
                    ("/revolver" | "/favicon.ico", Method::Get) => {
                        serve_file("revolver.png", request);
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
        s.spawn(move || {
           listen("192.168.1.248:6970", |out| {
                println!("Connection received");
                move |msg| {
                    out.send("pog")
                }
           }); 
        });
        s.spawn(move || {
            println!("{:?}", config.gameplay.cards);
            let mut player1 = Player {name: "pogger".to_string(), role: Roles::Deputy, health: 1i8, character: Characters::SlabTheKiller, weapon: Weapons::Schofield, cards: vec![Cards::Bang], attributes: HashSet::new()};
            player1.attributes.insert(Attributes::ExtraDistance);
            println!("{:?}", player1);
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
    //let file = File::options().write(true).open(&file(file_name)); 
    fs::write(&file(file_name), content.as_bytes());
}
fn append_to_file(file_name: &str, content: String) {
    let mut file = File::options()
        .append(true)
        .create(true)
        .open(&file(file_name))
        .unwrap();
    file.write_all(content.as_bytes());
}
fn serve_file(file_name: &str, request: Request) {
    let path = file(file_name);
    let file = File::open(&path).unwrap_or_else(|err| {
        eprintln!("ERROR: File - {err}");
        File::create(&path).unwrap()
    });
    request.respond(Response::from_file(file));
}
fn render_html(file_name: &str, mut tera: Tera, substitution: Vec<(&str, &str)>) -> String {
    let file_content = read_file(file_name);
    tera.build_inheritance_chains();
    let mut context = Context::new();
    for pair in substitution {
        let (key, value) = pair;
        context.insert(key, value);
    }
    let rendered = tera.render("html", &context).unwrap_or_else(|err| {
        eprintln!("ERROR: Tera - {err}");
        file_content
    });
    rendered
}
fn get_request_body(request: &mut Request) -> String {
    let mut buffer: String = Default::default();
    let _ = request.as_reader().read_to_string(&mut buffer);
    buffer
}
fn parse_config() -> Result<Table, de::Error> {
    let content = read_to_string(CONFIG_FILE).unwrap_or_else(|err| {
        eprintln!("ERROR  File - {err}");
        "".to_string()
    });
    let value = content.parse::<Table>();
    value
}
fn set_config(table: Table) -> Config {
    let content = read_to_string(CONFIG_FILE).unwrap_or_else(|err| {
        eprintln!("ERROR: File - {err}");
        "".to_string()
    });
    let config = toml::from_str::<Config>(&content).unwrap();
    println!("{:?}", config);
    config
}
