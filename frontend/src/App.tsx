import { gql, useQuery } from '@apollo/client';
import React from 'react';
import { Link, Route, HashRouter as Router, Switch } from 'react-router-dom';
import './App.css';
import Guild from './routes/Guild';

function App() {
    return (
        <Router>
            <div>
                <nav>
                    <ul>
                        <li>
                            <Link to="/">Home</Link>
                        </li>
                    </ul>
                </nav>

                {/* A <Switch> looks through its children <Route>s and
            renders the first one that matches the current URL. */}
                <Switch>
                    <Route path="/:guild_id" exact>
                        <Guild />
                    </Route>
                    <Route path="/:guild_id/:channel_id" exact>
                        {/* <Channel /> */}
                    </Route>
                    <Route path="/" exact>
                        <div>
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
