import { gql, useQuery } from "@apollo/client";
import React from "react";
import { Link, useParams } from "react-router-dom";
import { GetGuild, GetGuildVariables } from "./__generated__/GetGuild";

const GET_GUILD = gql`
    query GetGuild($guild_id: String!) {
        guild(id: $guild_id) {
            name,
            id,
            icon
            banner
            owner {
                id
                name
                avatar
                color
                nick
                discriminator
            }
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
    let { guild_id } = useParams<{ guild_id: string }>();
    let { loading, error, data, refetch } = useQuery<GetGuild, GetGuildVariables>(GET_GUILD, { variables: { guild_id } });

    if (loading) return <div>Loading...</div>;
    if (error) return <div>Error :(<br /><pre>{JSON.stringify(error, undefined, 4)}</pre></div>

    // Safety: I can guarantee, at this state that data is not null since it is only allowed to be null if there is an error, which would be caught
    let guild = data!.guild;

    if (guild === null) {
        return (
            <div>
                <h1>The guild with id {guild_id} does not exist or is not available to the bot</h1>
            </div>
        )
    }

    return (
        <div>
            <Link to="../">{"<"} Back</Link>
            <button onClick={() => refetch()}>Refresh</button>
            <table>
                <thead>
                    <tr>
                        <th>Property</th>
                        <th>Value</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>Guild name:</td>
                        <td>{guild.name}</td>
                    </tr>
                    <tr>
                        <td>Guild id:</td>
                        <td>{guild.id}</td>
                    </tr>
                    <tr>
                        <td>Guild owner:</td>
                        <td style={{ color: `#${guild.owner?.color?.toString(16) || "000000"}`, fontWeight: "bold" }}>{guild.owner === null ? "Could not be found" : `${guild.owner.nick || guild.owner.name}#${guild.owner.discriminator}`}</td>
                    </tr>
                    <tr>
                        <td>Voice channels:</td>
                        <td>
                            <table>
                                <thead>
                                    <tr>
                                        <th>Name</th>
                                        <th>Id</th>
                                        <th>Can operate</th>
                                        <th>User Limit</th>
                                        <th>Users</th>

                                    </tr>
                                </thead>
                                <tbody>
                                    {guild.voiceChannels.map((vc, i) => (
                                        <tr key={i}>
                                            <td>{vc.name}</td>
                                            <td>{vc.id}</td>
                                            <td>{vc.canOperate ? "true" : "false"}</td>
                                            <td>{vc.userLimit}</td>
                                            <td>{vc.states.length}</td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </td>
                    </tr>
                    <tr>
                        <td>Guild name:</td>
                        <td>{guild.name}</td>
                    </tr>
                </tbody>
            </table>

            <pre>
                {JSON.stringify(data, undefined, 4)}
            </pre>
        </div >
    );
}