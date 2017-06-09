import * as React from "react";
import * as ReactDOM from "react-dom";
import Flexbox from 'flexbox-react';

import * as T from './PTTypes';

export function renderPlayerUI(elmApp: any, [id, playerID, currentScene, data]: [string, T.PlayerID, T.SceneID | undefined, any]) {
  let element = document.getElementById(id);
  console.log("[renderPlayerUI] Rendering Player component from Elm", id, element, playerID, currentScene);
  let app = T.decodeApp.decodeAny(data);
  ReactDOM.render(
    <PlayerUI app={app} playerID={playerID} currentScene={currentScene} />,
    element
  );
}

class PlayerUI extends React.Component<
  { playerID: T.PlayerID; currentScene: string | undefined; app: T.App; },
  undefined> {

  render(): JSX.Element {
    console.log("[PlayerUI:render]");
    return <div style={{ display: "flex", flexDirection: "column" }}>
      <div>Player: {this.props.playerID}</div>
      <PlayerCreatures playerID={this.props.playerID} app={this.props.app} />
    </div>;
  }
}

class PlayerCreatures extends React.Component<
  { playerID: T.PlayerID; app: T.App; },
  undefined> {

  creatureCard(cid: T.CreatureID): JSX.Element {
    return <CreatureCard key={cid} app={this.props.app} creature_id={cid} />;
  }

  render(): JSX.Element {
    let cids = this.props.app.players[this.props.playerID].creatures;
    return <div>Player creatures!
      {cids.map(this.creatureCard.bind(this))}
    </div>
  }
}

class CreatureCard extends React.Component<{ creature_id: T.CreatureID; app: T.App }, undefined> {
  render(): JSX.Element {
    let creature = T.getCreature(this.props.app, this.props.creature_id);
    if (!creature) {
      return <div>Creature {this.props.creature_id} not found</div>;
    } else {

      return <div
        style={{
          display: "flex", flexDirection: "column",
          width: "300px",
          borderRadius: "10px", border: "1px solid black",
          padding: "3px"
        }}>
        <div><strong>{creature.name}</strong> {classIcon(creature)}</div>
        <div style={{ display: "flex" }}>
          <CreatureIcon app={this.props.app} creature={creature} />
          {/*, hbox (List.map conditionIcon (Dict.values creature.conditions))
      ] ++ extras*/}
        </div>
      </div>;
    }
  }
}

function classIcon(creature: T.Creature): string {
  switch (creature.class_) {
    case "cleric": return "💉";
    case "rogue": return "🗡️";
    case "ranger": return "🏹";
    case "creature": return "🏃";
    case "baddie": return "👹";
    default: return ""
  }
}

function CreatureIcon(props: { app: T.App, creature: T.Creature }): JSX.Element | null {
  let squareStyle = {width: "50px", height: "50px", borderRadius: "10px", border: "solid 1px black"};
  if (props.creature.portrait_url !== "") {
    return <img src={props.creature.portrait_url}
      style={squareStyle} />
  } else {
    let class_ = props.app.current_game.classes[props.creature.class_];
    let color;
    if (class_) {
      color = class_.color;
    } else {
      color = "red";
    }
    return <div style={{backgroundColor: color, ...squareStyle}}>{props.creature.name}</div>
  }
};
