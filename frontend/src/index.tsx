import { ApolloClient, ApolloProvider, InMemoryCache } from "@apollo/client";
import hash from "object-hash";
import React from "react";
import ReactDOM from "react-dom";
import App from "./App";
import { BACKEND_GRAPHQL_URL } from "./constants";
import * as serviceWorker from "./serviceWorker";
import { GlobalStyle } from "./style";

/** The client to use for apollo */
const CLIENT = new ApolloClient({
    cache: new InMemoryCache({
        dataIdFromObject: o => hash(o), // tslint:disable-line: no-unnecessary-callback-wrapper
    }),
    credentials: "include",
    headers: {
        Accept: "application/json"
    },
    name: "stfu",
    uri: BACKEND_GRAPHQL_URL
});

ReactDOM.render(
    <React.StrictMode>
        <GlobalStyle />
        <ApolloProvider client={CLIENT}>
            <App />
        </ApolloProvider>
    </React.StrictMode>,
    document.getElementById("root")
);

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister();
