import * as React from "react";
import * as ReactDOM from "react-dom";
import Flexbox from 'flexbox-react';

import * as T from './PTTypes';

export function renderPlayers(app: any, [id, currentScene, data]: [string, string, any]) {
  let onSetScene = (pid: T.PlayerID, scene: T.SceneID | null) =>
    app.ports.playersSetScene.send([pid, scene]);
  let onGrantCreatures = (pid: T.PlayerID) =>
    app.ports.playersGrantCreatures.send(pid);
  let element = document.getElementById(id);
  console.log("[renderPlayers] loading Players component", id, element, currentScene, data);
  ReactDOM.render(
    <Players data={data} currentScene={currentScene}
      onSetScene={onSetScene}
      onGrantCreatures={onGrantCreatures} />,
    element
  );
}

interface PlayersProps {
  currentScene: string | undefined;
  data: any;
  onSetScene: (pid: T.PlayerID, scene: T.SceneID | null) => void;
  onGrantCreatures: (pid: T.PlayerID) => void;
};

class Players extends React.Component<PlayersProps, undefined> {

  setSceneButton(pid: T.PlayerID, text: string, scene: T.SceneID | null): JSX.Element {
    return <button onClick={() => this.props.onSetScene(pid, scene)}>{text}</button >
  }

  playerCreatures(app: T.App, player: T.Player): JSX.Element {
    return <Flexbox flexDirection="column">
      <ul>
        {player.creatures.map((cid) => {
          return <li>{app.current_game.creatures[cid].name}</li>;
        })}
      </ul>
    </Flexbox>
  }

  render(): JSX.Element {
    let app = T.decodeApp.decodeAny(this.props.data);
    return <Flexbox flexDirection="column">{
      Object.keys(app.players).map((pid) => {
        let player = app.players[pid];
        let sceneButtons = [];
        if (player.scene) {
          sceneButtons.push(this.setSceneButton(pid, "Remove from Scene", null));
        }
        if (this.props.currentScene && player.scene !== this.props.currentScene) {
          sceneButtons.push(this.setSceneButton(pid, "Move to this scene", this.props.currentScene));
        }

        return <Flexbox alignItems="center" justifyContent="space-between" key={pid}>
          <div>{pid}</div>
          {this.playerCreatures(app, player)}
          {sceneButtons}
          <button onClick={() => this.props.onGrantCreatures(pid)}>Grant creatures</button>
        </Flexbox>;
      })
    }</Flexbox>;
  }
}