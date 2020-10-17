import React from "react";
import { HashRouter as Router, Link, Route, Switch } from "react-router-dom";
import Channel from "./routes/Channel";
import Guild from "./routes/Guild";

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
                        <div>
                            <div>/ Home</div>

                            whats poppin

                            <div>
                                <input type="text" value={"708811110928744578"} />
                                <Link to={"/708811110928744578"}>Go to guild</Link>
                            </div>
                        </div>
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
