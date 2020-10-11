import { gql, useQuery } from '@apollo/client';
import React from 'react';
import './App.css';

const ME = gql`
    query BotMeta {
        me {
            name
            id
            discriminator
        }
    }
`;

function App() {
    let { loading, error, data } = useQuery(ME);

    if (loading) return <div>Loading...</div>;

    return (
        <div >
            whats poppin
        </div>
    );
}

export default App;
