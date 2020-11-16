import { FetchResult, gql, useMutation, useQuery } from "@apollo/client";
import React from "react";
import { Link, useParams } from "react-router-dom";
import ErrorScreen from "../components/Error";
import LazyAnimateImage from "../components/LazyAnimateImage";
import { LoadingIcon, LoadingScreen } from "../components/Loading";
import { getAvatar } from "../utils";
import { GetChannel, GetChannelVariables, GetChannel_guild, GetChannel_guild_voiceChannel } from "./__generated__/GetChannel";
import { MuteAll, MuteAllVariables } from "./__generated__/MuteAll";
import { UnmuteAll, UnmuteAllVariables } from "./__generated__/UnmuteAll";

/** The graphql query to get the specific channel */
const GET_CHANNEL = gql`
    query GetChannel($guild_id: String!, $channel_id: String!) {
        guild(id: $guild_id) {
            name
            id
            voiceChannel(id: $channel_id) {
                botMissingPermissions
                userMissingPermissions
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

/** The graphql query to mute all in a channel */
const MUTE_ALL = gql`
    mutation MuteAll($channel_id: String!, $guild_id: String!) {
        mute(channelId: $channel_id, guildId: $guild_id)
    }
`;

/** The graphql query to unmute all in a channel */
const UNMUTE_ALL = gql`
    mutation UnmuteAll($channel_id: String!, $guild_id: String!) {
        unmute(channelId: $channel_id, guildId: $guild_id)
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
            pollInterval: 60000,
            variables: {
                channel_id,
                guild_id,
            },
        }
    );
    const [muteAll] = useMutation<MuteAll, MuteAllVariables>(MUTE_ALL, {
        notifyOnNetworkStatusChange: true,
        variables: {
            channel_id,
            guild_id
        }
    });
    const [unmuteAll] = useMutation<UnmuteAll, UnmuteAllVariables>(UNMUTE_ALL, {
        notifyOnNetworkStatusChange: true,
        variables: {
            channel_id,
            guild_id
        }
    });

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
                <ChannelInfo guild={guild} channel={channel} refetch={refetch_no_await} muteAll={muteAll} unmuteAll={unmuteAll} />
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
    /** The mute all function to unmute all */
    muteAll(): Promise<FetchResult<MuteAll>>;
    /** The unmute all function to unmute all */
    unmuteAll(): Promise<FetchResult<UnmuteAll>>;
}

/** The information on a specific channel */
function ChannelInfo({ guild, channel, refetch, muteAll, unmuteAll }: IChannelInfoProps) {
    let operable = channel.botMissingPermissions === null && channel.userMissingPermissions === null;
    return (
        <div>
            <div>/ <Link to="/">Home</Link> / <Link to={`/${guild.id}`}>{guild.name}</Link> / {channel.name} </div>
            <button onClick={refetch}>Refresh</button>

            <div>
                {channel.botMissingPermissions === null ? undefined : <div>The bot is unable to perform actions on this channel. You are missing the following permissions on the channel: {channel.botMissingPermissions?.join(", ")}</div>}
                {channel.userMissingPermissions === null ? undefined : <div>You are unable to perform actions on this channel. You are missing the following permissions on this channel: {channel.userMissingPermissions?.join(", ")}</div>}
                <button onClick={muteAll.bind(muteAll)} disabled={!operable}>Mute</button>
                <button onClick={unmuteAll.bind(unmuteAll)} disabled={!operable}>Unmute</button>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Position</th>
                        <th>Bot Missing Permissions</th>
                        <th>User Missing Permissions</th>
                        <th>User Limit</th>
                        <th>User Count</th>
                        <th>Users</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>{channel.name}</td>
                        <td>{channel.position}</td>
                        <td>{channel.botMissingPermissions?.join(", ")}</td>
                        <td>{channel.userMissingPermissions?.join(", ")}</td>
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
                                    {channel.states.map((s, i) => {
                                        const source = (hover: boolean) => `${getAvatar(s.member, hover)}?size=64`;

                                        return (
                                            <tr key={i}>
                                                <td style={{ color: `#${s.member.color === null ? "000000" : s.member.color.toString(16)}`, fontWeight: "bold" }}>{s.member.name}#{s.member.discriminator}{s.member.nick === null ? undefined : ` (${s.member.nick})`}</td>
                                                <td><LazyAnimateImage source={source} alt={`${s.member.name}'s avatar`} /></td>
                                                <td>{s.member.id}</td>
                                                <td>{s.mute.toString()}</td>
                                                <td>{s.deaf.toString()}</td>
                                                <td>{s.selfMute.toString()}</td>
                                                <td>{s.selfDeaf.toString()}</td>
                                            </tr>
                                        );
                                    })}
                                </tbody>
                            </table>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div >
    );
}
