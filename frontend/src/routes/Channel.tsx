import { gql, useQuery } from "@apollo/client";
import React from "react";
import { Link, useParams } from "react-router-dom";
import ErrorScreen from "../components/Error";
import { LoadingIcon, LoadingScreen } from "../components/Loading";
import { getAvatar } from "../utils";
import { GetChannel, GetChannelVariables, GetChannel_guild, GetChannel_guild_voiceChannel } from "./__generated__/GetChannel";

/** The graphql query to get the specific channel */
const GET_CHANNEL = gql`
    query GetChannel($guild_id: String!, $channel_id: String!) {
        guild(id: $guild_id) {
            name
            id
            voiceChannel(id: $channel_id) {
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
                    channelId
                    deaf
                    mute
                    selfDeaf
                    selfMute
                    id
                    member {
                        avatar
                        bot
                        color
                        discriminator
                        id
                        joinedAt
                        mute
                        name
                        nick
                    }
                }
                userLimit
            }
        }
    }
`;

/** The parameters for the channel */
interface IParams {
    /** The parent guild's id */
    guild_id: string;
    /** The channel id to view */
    channel_id: string;
}

/** The voice channel view */
export default function Channel() {
    const { guild_id, channel_id } = useParams<IParams>();
    const { loading, error, data, refetch } = useQuery<GetChannel, GetChannelVariables>(
        GET_CHANNEL,
        {
            notifyOnNetworkStatusChange: true,
            pollInterval: 15000,
            variables: {
                channel_id,
                guild_id,
            },
        }
    );
    const refetch_no_await = () => { refetch().catch((e) => console.error(e)); };

    if (loading && data === undefined) {
        return (
            <LoadingScreen refetch={refetch_no_await} />
        );
    } else if (error !== undefined) {
        return (
            <ErrorScreen error={error} refetch={refetch_no_await} />
        );
    } else {
        // Safety: I can guarantee, at this state that data is not null since it is only allowed to be null if there is an error, which would be caught
        // tslint:disable-next-line:no-non-null-assertion
        const guild = data!.guild;

        if (guild === null) {
            return (
                <div>
                    <h1>The guild with id {guild_id} does not exist or is not available to the bot</h1>
                </div>
            );
        }

        const channel = guild.voiceChannel;

        if (channel === null) {
            return (
                <div>
                    <h1>The channel with id {channel_id} does not exist, is not a voice channel, or is not available to the bot</h1>
                </div>
            );
        }

        return (
            <>
                {loading ? <LoadingIcon /> : undefined}
                <ChannelInfo guild={guild} channel={channel} refetch={refetch_no_await} />
                <pre>
                    {JSON.stringify(data, undefined, 4)}
                </pre>
            </>
        );
    }
}

/** The props for the GuildInfo component */
interface IChannelInfoProps {
    /** The guild to view */
    guild: GetChannel_guild;
    /** The voice channel to view */
    channel: GetChannel_guild_voiceChannel;
    /** The refresh function to query a refresh */
    refetch(): void;
}

/** The information on a specific channel */
function ChannelInfo({ guild, channel, refetch }: IChannelInfoProps) {
    return (
        <div>
            <div>/ <Link to="/">Home</Link> / <Link to={`/${guild.id}`}>{guild.name}</Link> / {channel.name} </div>
            <button onClick={refetch}>Refresh</button>

            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Position</th>
                        <th>Can operate</th>
                        <th>User Limit</th>
                        <th>User Count</th>
                        <th>Users</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>{channel.name}</td>
                        <td>{channel.position}</td>
                        <td>{channel.canOperate.toString()}</td>
                        <td>{channel.userLimit}</td>
                        <td>{channel.states.length}</td>
                        <td>
                            <table>
                                <thead>
                                    <tr>
                                        <th>Name</th>
                                        <th>Avatar</th>
                                        <th>Id</th>
                                        <th>Server Mute</th>
                                        <th>Server Deaf</th>
                                        <th>Self Mute</th>
                                        <th>Self Deaf</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {channel.states.map((s, i) => (
                                        <tr key={i}>
                                            <td style={{ color: `#${s.member.color === null ? "000000" : s.member.color.toString(16)}`, fontWeight: "bold" }}>{s.member.name}#{s.member.discriminator}{s.member.nick === null ? undefined : ` (${s.member.nick})`}</td>
                                            {/* FIXME: deal with no avatar */}
                                            <td><img src={`${getAvatar(s.member)}?size=64`} alt={`${s.member.name}'s avatar`}/></td>
                                            <td>{s.member.id}</td>
                                            <td>{s.mute.toString()}</td>
                                            <td>{s.deaf.toString()}</td>
                                            <td>{s.selfMute.toString()}</td>
                                            <td>{s.selfDeaf.toString()}</td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div >
    );
}