module Model exposing (..)

import Dict
import Http
import Keyboard
import Keyboard.Key as Key
import Time

import Types as T


type alias GotCreatures = List T.CreatureID -> Msg

subscriptions : Model -> Sub Msg
subscriptions model =
  let ticks =
        case model.showingMovement of
          ShowingMovement _ _ -> Time.every (Time.second / 4) Tick
          _ -> Sub.none
      handleKey key =
        case (Key.fromCode key) of
          Key.Up -> MapPan Up
          Key.Down -> MapPan Down
          Key.Left -> MapPan Left
          Key.Right -> MapPan Right
          _ -> NoMsg
          -- Key.Add -> MapZoom In
          -- Key.Subtract -> MapZoom Out
      keys = Keyboard.downs handleKey
  in Sub.batch [ticks, keys]

type Msg
    = Start
    | Batch (List Msg)
    | SetFocus Focus
    | SetSecondaryFocus SecondaryFocus
    | SetModal Modal
    | MorePlease
    | PollApp
    | ReceivedAppUpdate (Result Http.Error T.App)
    | AppUpdate (Result Http.Error T.App)
    | ShowError String
    | ClearError
    | SetPlayerID T.PlayerID
    | RegisterPlayer

    | ToggleTerrain T.Point3

    | CommandComplete (Result Http.Error T.RustResult)
    | ToggleSelectedCreature T.CreatureID
    | SelectAbility T.SceneID T.CreatureID T.AbilityID
    | CancelAbility
    | GotTargetOptions (Result Http.Error (List T.PotentialTarget))
    | CombatAct T.AbilityID T.DecidedTarget
    | ActCreature T.SceneID  T.CreatureID T.AbilityID T.DecidedTarget
    | RequestMove MovementRequest
    | CancelMovement
    | PathCurrentCombatCreature T.Point3
    | PathCreature T.SceneID T.CreatureID T.Point3
    | SetCreaturePos T.SceneID T.CreatureID T.Point3
    | GetMovementOptions T.SceneID T.Creature
    | GetCombatMovementOptions
    | GotCombatMovementOptions (Result Http.Error (List T.Point3))
    | GotMovementOptions T.Creature (Result Http.Error (List T.Point3))
    | SelectCreatures (List T.CreatureID) GotCreatures String
    | DoneSelectingCreatures
    | CancelSelectingCreatures
    | ToggleMoveAnywhere
    | Tick Time.Time
    | SendCommand T.GameCommand
    | GetSavedGames (List String -> Msg)
    | GotSavedGames (Result Http.Error (List String))
    | SaveGame String
    | SavedGame (Result Http.Error ())
    | LoadGame String
    | SetCreatureNote T.CreatureID String
    | MapZoom MapInOut
    | MapPan Direction
    | ToggleCollapsed String
    | SelectView String String

    | NoMsg

type MapInOut
  = In | Out

type Direction
  = Left
  | Right
  | Up
  | Down

defaultModel : ProgramFlags -> Model
defaultModel flags =
  { app = Nothing
  , selectedAbility = Nothing
  , selectingCreatures = Nothing
  , moving = Nothing
  , error = ""
  , playerID = Nothing
  , potentialTargets = []
  , moveAnywhere = False
  , showingMovement = NotShowingMovement
  , creatureNotes = Dict.empty
  , rpiURL = flags.rpi
  , gridSize = 60
  , gridOffset = {x = -15, y = 10}
  , collapsed = Dict.empty
  , selectedViews = Dict.empty
  , focus = NoFocus
  , secondaryFocus = Focus2None
  , modal = NoModal
  , gettingSavedGames = Nothing
  }

type alias Model =
  { app : Maybe T.App
  , selectedAbility : Maybe (T.SceneID, T.CreatureID, T.AbilityID)
  -- Creatures which have been selected for combat
  , selectingCreatures : Maybe (List T.CreatureID, List T.CreatureID, GotCreatures, String)
  , error: String
  , moving: Maybe MovementRequest
  , playerID : Maybe T.PlayerID
  , potentialTargets: List T.PotentialTarget
  , showingMovement: MovementAnimation
  , creatureNotes : Dict.Dict T.CreatureID String
  , moveAnywhere : Bool
  , rpiURL : String
  -- gridSize: how many SQUARE METERS to show
  , gridSize : Int
  -- gridOffset: offset in METERS
  , gridOffset : {x : Int, y: Int}
  , collapsed : Dict.Dict String Bool
  , selectedViews : Dict.Dict String String
  , focus: Focus
  , secondaryFocus: SecondaryFocus
  , modal: Modal
  , gettingSavedGames: Maybe (List String -> Msg)
  }
  
type Focus
  = NoFocus
  | Scene String
  | EditingMap T.FolderPath T.Map
  | PreviewMap T.MapID

type SecondaryFocus
  = Focus2None
  | Focus2Creature T.FolderPath T.CreatureID
  | Focus2Note T.FolderPath String T.Note
  | Focus2Map T.FolderPath T.MapID
  | Focus2Scene T.FolderPath T.SceneID

type Modal
  = NoModal
  | CreateFolder CreatingFolder
  | CreateCreature PendingCreature
  | CreateScene CreatingScene
  | CreateMap CreatingMap
  | MoveFolderItem MovingFolderItem
  | RenameFolder RenamingFolder
  | SelectCreaturesFromCampaign SelectingCreatures
  | ModalLoadGame (List String)
  | ModalSaveGame SavingGame
  | ModalEditCreature EditingCreature

type alias SavingGame = {existing: List String, newGame: String}
type alias CreatingFolder = {parent: T.FolderPath , child: String}
type alias CreatingScene = {path: T.FolderPath , scene: T.SceneCreation}
type alias CreatingMap = {path: T.FolderPath, name: String}
type alias MovingFolderItem = {src: T.FolderPath, item: T.FolderItemID, dst: T.FolderPath}
type alias RenamingFolder = {path: T.FolderPath, newName: String}
type alias SelectingCreatures = {cb: GotCreatures, reason: String, selectedCreatures : List T.CreatureID}
type alias EditingCreature = {cid: T.CreatureID, note: String, portrait_url: String}

devFlags : ProgramFlags
devFlags = {rpi = "http://localhost:1337/"}

type alias ProgramFlags =
  { rpi : String }

type alias PendingCreature = {name: Maybe T.CreatureID, class: Maybe String, path: T.FolderPath}

type MovementAnimation
  = ShowingMovement (List T.Point3) (List T.Point3) -- first is what's been shown so far, second is what's left to animate
  | DoneShowingMovement (List T.Point3)
  | NotShowingMovement


type alias MovementRequest = {
  max_distance: T.Distance,
  movement_options: List T.Point3,
  -- This field is a Just when we're performing out-of-combat movement.
  -- It's Nothing for in-combat movement, because in-combat movement is only for the current
  -- creature.
  ooc_creature: Maybe T.Creature
}

type alias FolderItem =
  { key: T.FolderItemID
  , path: T.FolderPath
  , prettyName : String
  }

getScene : Model -> String -> Maybe T.Scene
getScene model name =
  case model.app of
    Just app -> Dict.get name app.current_game.scenes
    Nothing -> Nothing

getMap : Model -> T.Map
getMap model =
  case model.focus of
    EditingMap path map -> map
    PreviewMap name ->
      model.app
      |> Maybe.andThen (getMapNamed name)
      |> Maybe.withDefault T.emptyMap
    Scene name ->
      getMapForScene model name
    NoFocus -> T.emptyMap

getMapForScene : Model -> String -> T.Map
getMapForScene model name =
  getScene model name
  |> Maybe.andThen (\scene -> model.app |> Maybe.andThen (getMapNamed scene.map))
  |> Maybe.withDefault T.emptyMap


getMapNamed : String -> T.App -> Maybe T.Map
getMapNamed name app =
  Dict.get name app.current_game.maps

tryGetMapNamed : String -> T.App -> T.Map
tryGetMapNamed name app = getMapNamed name app |> Maybe.withDefault T.emptyMap
