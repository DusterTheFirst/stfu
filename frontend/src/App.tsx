import { gql, useQuery } from '@apollo/client';
import React from 'react';
import './App.css';
import { BotMeta } from './__generated__/BotMeta';

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
    let { loading, error, data } = useQuery<BotMeta>(ME);

    if (loading) return <div>Loading...</div>;
    if (error) return <div>Error :(<br /><pre>{JSON.stringify(error, undefined, 4)}</pre></div>
    if (data === undefined) throw new Error("Undefined state");

    return (
        <div >
            whats poppin

            <pre>
                {JSON.stringify(data, undefined, 4)}
            </pre>
        </div>
    );
}

export default App;
