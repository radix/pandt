import * as React from "react";
import * as ReactDOM from "react-dom";
import Flexbox from 'flexbox-react';

import * as PTTypes from './PTTypes';

export function renderHistory([id, data]: [string, Array<Array<[any, Array<any>]>>]) {
  console.log("Rendering History", id, data);
  ReactDOM.render(
    <History data={data} />,
    document.getElementById(id)
  );
}

class History extends React.Component<{ data: any }, any> {
  render(): JSX.Element {
    let snaps = PTTypes.decodeAppSnapshots.decodeAny(this.props.data);
    return <Flexbox flexDirection="column">{
      snaps.map(
        ({ snapshot, logs }) =>
          logs.map((log, i) => <Flexbox key={i}>{this.gameLog(log)}</Flexbox>)
      )
    }</Flexbox>;
  }

  gameLog(log: PTTypes.GameLog): JSX.Element {
    switch (log.t) {
      case "AttributeCheckResult":
        return <Flexbox>
          <div>Creature ID: {log.cid}</div>
          <div>Success? {log.success.toString()}</div>
        </Flexbox>;
      case "CreateFolder":
        return <Flexbox><div>Created Folder</div><div>{log.path}</div></Flexbox>;
      case "RenameFolder":
        return <Flexbox>Renamed Folder</Flexbox>
      case "DeleteFolder":
        return <Flexbox>Deleted folder</Flexbox>
      case "MoveFolderItem":
        return <Flexbox>Moved folder item</Flexbox>
      case "CreateNote":
        return <Flexbox>Created note {log.note.name}</Flexbox>
      case "EditNote":
        return <Flexbox>Edited note {log.name}</Flexbox>
      case "DeleteNote":
        return <Flexbox>Deleted note {log.name}</Flexbox>
      case "CreateScene":
        return <Flexbox>Created scene {log.scene}</Flexbox>
      case "EditScene":
        return <Flexbox>Edited scene {log.scene.name}</Flexbox>
      case "DeleteScene":
        return <Flexbox>Deleted a scene</Flexbox>
      case "CreateMap":
        return <Flexbox>Created a map {log.map.name}</Flexbox>
      case "EditMap":
        return <Flexbox>Edited a map {log.map.name}</Flexbox>
      case "DeleteMap":
        return <Flexbox>Deleted a map</Flexbox>
      case "SetCreaturePos":
        return <Flexbox>Set a creature position</Flexbox>
      case "PathCreature":
        return <Flexbox>Creature followed a path</Flexbox>
      case "CreateCreature":
        return <Flexbox>Created a creature {log.creature.name}</Flexbox>
      case "EditCreature":
        return <Flexbox>Edited a creature {log.creature.name}</Flexbox>
      case "DeleteCreature":
        return <Flexbox>Deleted a creature</Flexbox>
      case "StartCombat":
        return <Flexbox>Started combat</Flexbox>
      case "AddCreatureToCombat":
        return <Flexbox>Added a creature to combat</Flexbox>
      case "RemoveCreatureFromCombat":
        return <Flexbox>Removed a creature from combat</Flexbox>
      case "CombatLog":
        return combat_log(log.log)
      case "StopCombat":
        return <Flexbox>Combat stopped.</Flexbox>;
      case "CreatureLog":
        return creature_log(log.log)
      case "Rollback":
        return <Flexbox>Rolled back to {log.snapshot_index}/{log.log_index}</Flexbox>
    }
  }
}

function combat_log(log: PTTypes.CombatLog): JSX.Element {
  switch (log.t) {
    case "ConsumeMovement":
      return <noscript />;
    case "ChangeCreatureInitiative":
      return <Flexbox>Creature initiative changed</Flexbox>
    case "EndTurn":
      return <Flexbox>Creature's turn ended.</Flexbox>
    case "ForceNextTurn":
      return <Flexbox>Forced move to next creature in combat</Flexbox>
    case "ForcePrevTurn":
      return <Flexbox>Forced move to previous creature in combat</Flexbox>
    case "RerollInitiative":
      return <Flexbox>Rerolled initiative for all creatures</Flexbox>
  }
}

function creature_log(log: PTTypes.CreatureLog): JSX.Element {
  switch (log.t) {
    case "Damage":
      return <Flexbox>A creature took {log.hp} damage. Rolls: {JSON.stringify(log.rolls)}</Flexbox>
    case "Heal":
      return <Flexbox>A creature was healed for {log.hp}. Rolls: {JSON.stringify(log.rolls)}</Flexbox>
    case "GenerateEnergy":
      return <Flexbox>A creature received {log.energy} energy.</Flexbox>
    case "ReduceEnergy":
      return <Flexbox>A creature's energy was reduced by {log.energy}</Flexbox>
    case "ApplyCondition":
      return <Flexbox>A creature gained a condition</Flexbox>
    case "DecrementConditionRemaining":
      return <Flexbox>A condition was reduced in duration.</Flexbox>
    case "RemoveCondition":
      return <Flexbox>A condition was removed from a creature.</Flexbox>
  }
}

interface AttributeCheckResult { t: "AttributeCheckResult"; }

