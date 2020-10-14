import { gql, useQuery } from "@apollo/client";
import React from "react";
import { useParams } from "react-router-dom";
import { GetGuild, GetGuildVariables } from "./__generated__/GetGuild";

const GET_GUILD = gql`
    query GetGuild($id: String!) {
        guild(id: $id) {
            name,
            id,
            icon
            banner
            ownerId
            unavailable
            voiceChannels {
                canOperate
                category {
                    id
                    name
                    position
                }
                id
                name
                position
                states {
                    deaf
                    id
                    member {
                        id
                        name
                        avatar
                        color
                        nick
                    }
                    mute
                }
                userLimit
            }
        }
    }
`;

export default function Guild() {
    let { id } = useParams<{ id: string }>();
    let { loading, error, data } = useQuery<GetGuild, GetGuildVariables>(GET_GUILD, { variables: { id } });

    if (loading) return <div>Loading...</div>;
    if (error) return <div>Error :(<br /><pre>{JSON.stringify(error, undefined, 4)}</pre></div>
    if (data === undefined) throw new Error("Undefined state");

    return (
        <div>
            <pre>
                {JSON.stringify(data, undefined, 4)}
            </pre>
        </div>
    );
}