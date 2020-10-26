import React from "react";
import { HashRouter as Router, Route, Switch } from "react-router-dom";
import Channel from "./routes/Channel";
import Guild from "./routes/Guild";
import Overview from "./routes/Overview";

/** The main entry point for the app */
function App() {
    return (
        <Router>
            <div>
                {/* A <Switch> looks through its children <Route>s and
            renders the first one that matches the current URL. */}
                <Switch>
                    <Route path="/:guild_id" exact={true}>
                        <Guild />
                    </Route>
                    <Route path="/:guild_id/:channel_id" exact={true}>
                        <Channel />
                    </Route>
                    <Route path="/" exact={true}>
                        <Overview />
                    </Route>
                    <Route path="*">
                        <pre>404</pre>
                    </Route>
                </Switch>
            </div>
        </Router>
    );
}

export default App;
