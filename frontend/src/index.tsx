import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';
import * as serviceWorker from './serviceWorker';
import { GlobalStyle } from './style';
import { ApolloClient, ApolloProvider, InMemoryCache } from '@apollo/client';

const CLIENT = new ApolloClient({
    name: "stfu",
    uri: "http://localhost:8000/graphql",
    cache: new InMemoryCache()
});

ReactDOM.render(
    <React.StrictMode>
        <GlobalStyle />
        <ApolloProvider client={CLIENT}>
            <App />
        </ApolloProvider>
    </React.StrictMode>,
    document.getElementById('root')
);


// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister();
