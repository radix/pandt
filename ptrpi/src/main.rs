#![feature(slice_patterns)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate bus;
#[macro_use]
extern crate error_chain;
extern crate owning_ref;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

extern crate pandt;

use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time;

use bus::Bus;
use owning_ref::MutexGuardRefMut;

use rocket::State;
use rocket_contrib::Json;
use rocket::http::Method;

mod cors;
use cors::{PreflightCORS, CORS};

use pandt::types::{App, CreatureID, GameCommand, GameError, GameErrorEnum, Point3,
                   PotentialTargets, RPIApp, RPIGame, Runtime};


error_chain! {
  types { RPIError, RPIErrorKind, RPIResultExt; }

  links {
    GameError(GameError, GameErrorEnum);
  }
  foreign_links {
    JSONError(serde_json::error::Error);
    IOError(::std::io::Error);
    YAMLError(serde_yaml::Error);
  }

  errors {
    LockError(resource: String) {
      description("Some locked resource is poisoned. The application probably needs restarted.")
      display("The lock on {} is poisoned. The application probably needs restarted.", resource)
    }
    InsecurePath(name: String) {
      description("A path cannot contain certain characters or elements for security reasons.")
      display("The path {} is insecure.", name)
    }
  }
}

type PTResult<X> = Result<CORS<Json<X>>, RPIError>;

#[derive(Clone)]
struct PT {
  runtime: Arc<Mutex<Runtime>>,
  pollers: Arc<Mutex<bus::Bus<()>>>,
  saved_game_path: PathBuf,
}

impl PT {
  fn app(&self) -> MutexGuardRefMut<Runtime, App> {
    match self.runtime.lock() {
      Ok(g) => MutexGuardRefMut::new(g).map_mut(|rt| &mut rt.app),
      Err(poison) => MutexGuardRefMut::new(poison.into_inner()).map_mut(|rt| &mut rt.app),
    }
  }

  fn pollers(&self) -> Result<MutexGuard<bus::Bus<()>>, RPIError> {
    self.pollers.lock().map_err(|_| RPIErrorKind::LockError("pollers".to_string()).into())
  }
}

#[route(OPTIONS, "/")]
fn options_handler() -> PreflightCORS {
  CORS::preflight("*").methods(vec![Method::Options, Method::Post]).headers(vec!["Content-Type"])
}

#[get("/")]
fn get_app(pt: State<PT>) -> Result<CORS<String>, RPIError> {
  let app = pt.app();
  let result = serde_json::to_string(&RPIApp(&*app))?;
  Ok(CORS::any(result))
}

/// If the client is polling with a non-current app "version", then immediately return the current
/// App. Otherwise, wait 30 seconds for any new changes.
#[get("/poll/<snapshot_len>/<log_len>")]
fn poll_app(pt: State<PT>, snapshot_len: usize, log_len: usize) -> Result<CORS<String>, RPIError> {
  {
    let app = pt.app();
    if app.snapshots.len() != snapshot_len ||
      app.snapshots.back().map(|&(_, ref ls)| ls.len()).unwrap_or(0) != log_len
    {
      let result = serde_json::to_string(&RPIApp(&*app))?;
      return Ok(CORS::any(result));
    }
  }

  let mut reader = pt.pollers()?.add_rx();
  // this will either return a timeout or (); in any case we'll just return the App to the client.
  let _ = reader.recv_timeout(time::Duration::from_secs(30));
  get_app(pt)
}

#[post("/", format = "application/json", data = "<command>")]
fn post_app(command: Json<GameCommand>, pt: State<PT>) -> Result<CORS<String>, RPIError> {
  let json = {
    let mut app = pt.app();
    let game = app.perform_unchecked(command.0).map(|(g, l)| (RPIGame(g), l)).map_err(|e| format!("Error: {}", e));
    serde_json::to_string(&game)
  };
  pt.pollers()?.broadcast(());
  Ok(CORS::any(json?))
}

#[get("/combat_movement_options")]
fn combat_movement_options(pt: State<PT>) -> PTResult<Vec<Point3>> {
  let app = pt.app();
  Ok(CORS::any(Json(app.get_combat_movement_options()?)))
}

#[get("/movement_options/<scene_id>/<cid>")]
fn movement_options(pt: State<PT>, scene_id: String, cid: String) -> PTResult<Vec<Point3>> {
  let app = pt.app();
  let cid = cid.parse()?;
  let scene = scene_id.parse()?;
  Ok(CORS::any(Json(app.get_movement_options(scene, cid)?)))
}

#[get("/target_options/<scene_id>/<cid>/<abid>")]
fn target_options(pt: State<PT>, scene_id: String, cid: String, abid: String)
  -> PTResult<PotentialTargets> {
  let app = pt.app();
  let scene = scene_id.parse()?;
  let cid = cid.parse()?;
  let abid = abid.parse()?;
  Ok(CORS::any(Json(app.get_target_options(scene, cid, abid)?)))
}

#[route(OPTIONS, "/preview_volume_targets/<scene>/<actor_id>/<ability_id>/<x>/<y>/<z>")]
fn options_creatures_in_volume(scene: String, actor_id: String, ability_id: String, x: String, y: String, z: String)
  -> PreflightCORS {
  options_handler()
}

#[post("/preview_volume_targets/<scene_id>/<actor_id>/<ability_id>/<x>/<y>/<z>")]
fn preview_volume_targets(pt: State<PT>, scene_id: String, actor_id: String, ability_id: String, x: i16, y: i16, z: i16)
  -> PTResult<(Vec<CreatureID>, Vec<Point3>)> {
  let app = pt.app();
  let sid = scene_id.parse()?;
  let actor_id = actor_id.parse()?;
  let ability_id = ability_id.parse()?;
  let point = (x, y, z);
  Ok(CORS::any(Json(app.preview_volume_targets(sid, actor_id, ability_id, point)?)))
}

#[get("/saved_games")]
fn list_saved_games(pt: State<PT>) -> PTResult<Vec<String>> {
  let mut result = vec![];
  for mpath in fs::read_dir(&pt.saved_game_path)? {
    let path = mpath?;
    if path.file_type()?.is_file() {
      match path.file_name().into_string() {
        Ok(s) => result.push(s),
        Err(x) => println!("Couldn't parse filename as unicode: {:?}", x),
      }
    }
  }
  Ok(CORS::any(Json(result)))
}

#[post("/saved_games/<name>/load")]
fn load_saved_game(pt: State<PT>, name: String) -> Result<CORS<String>, RPIError> {
  let path = child_path(&pt.saved_game_path, name)?;
  let mut buffer = String::new();
  File::open(path)?.read_to_string(&mut buffer)?;
  let app = serde_yaml::from_str(&buffer)?;
  *(pt.app()) = app;
  get_app(pt)
}

#[post("/saved_games/<name>")]
fn save_game(pt: State<PT>, name: String) -> PTResult<()> {
  let new_path = child_path(&pt.saved_game_path, name)?;
  let yaml = serde_yaml::to_string(&*pt.app())?;
  File::create(new_path)?.write_all(yaml.as_bytes())?;
  Ok(CORS::any(Json(())))
}

fn child_path(parent: &PathBuf, name: String) -> Result<PathBuf, RPIError> {
  if name.contains("/") || name.contains(":") || name.contains("\\") {
    bail!(RPIErrorKind::InsecurePath(name));
  }
  let new_path = parent.join(name.clone());
  for p in &new_path {
    if p == "." || p == ".." {
      bail!(RPIErrorKind::InsecurePath(name));
    }
  }
  Ok(new_path)
}

fn load_app_from_path(filename: &Path) -> App {
  let mut appf = File::open(filename).unwrap();
  let mut apps = String::new();
  appf.read_to_string(&mut apps).unwrap();
  serde_yaml::from_str(&apps).unwrap()
}

fn main() {
  let game_dir = env::args().nth(1).unwrap_or_else(|| {
    env::current_dir()
      .expect("couldn't get curdir")
      .into_os_string()
      .into_string()
      .expect("Couldn't parse curdir as string")
  });
  let game_dir = PathBuf::from(game_dir);
  let initial_file = env::args().nth(2).unwrap_or("samplegame.yaml".to_string());

  let app: App = load_app_from_path(game_dir.join(initial_file).as_path());
  let runtime = Runtime {app};

  let pt = PT {
    runtime: Arc::new(Mutex::new(runtime)),
    pollers: Arc::new(Mutex::new(Bus::new(1000))),
    saved_game_path: fs::canonicalize(game_dir).expect("Couldn't canonicalize game dir"),
  };

  rocket::ignite()
    .mount(
      "/",
      routes![
        get_app,
        poll_app,
        options_handler,
        post_app,
        combat_movement_options,
        movement_options,
        target_options,
        options_creatures_in_volume,
        preview_volume_targets,
        list_saved_games,
        load_saved_game,
        save_game,
      ],
    )
    .manage(pt)
    .launch();
}

#[cfg(test)]
mod test {
  use std::path::Path;

  #[test]
  fn load_samplegame_yaml() {
    ::load_app_from_path(Path::new("sample_games/samplegame.yaml"));
  }
}
