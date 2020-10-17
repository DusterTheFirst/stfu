import { gql, useQuery } from "@apollo/client";
import React, { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import ErrorScreen from "../components/Error";
import { LoadingIcon, LoadingScreen } from "../components/Loading";
import { getGuildIcon } from "../utils";
import {
    GetGuild, GetGuildVariables, GetGuild_guild, GetGuild_guild_voiceChannels as VoiceChannel, GetGuild_guild_voiceChannels_category as ChannelCategory
} from "./__generated__/GetGuild";

/** The graphql query to get information about the specific guild */
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
                    channelId
                    member {
                        id
                        name
                        avatar
                        color
                        nick
                        discriminator
                    }
                    mute
                }
                userLimit
            }
        }
    }
`;

/** The parameters for the guild url */
interface IParams {
    /** The guild id to view */
    guild_id: string;
}

/** The page for guild information */
export default function Guild() {
    const { guild_id } = useParams<IParams>();
    const { loading, error, data, refetch } = useQuery<GetGuild, GetGuildVariables>(GET_GUILD, { variables: { guild_id }, notifyOnNetworkStatusChange: true });
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

        return (
            <>
                {loading ? <LoadingIcon /> : undefined}
                <GuildInfo guild={guild} refetch={refetch_no_await} />
                <pre>
                    {JSON.stringify(data, undefined, 4)}
                </pre>
            </>
        );
    }
}

/** The props for the GuildInfo component */
interface IGuildInfoProps {
    /** The guild to view */
    guild: GetGuild_guild;
    /** The refresh function to query a refresh */
    refetch(): void;
}

/** The map for the voice chats */
type VCMap = Map<ChannelCategory | null, VoiceChannel[]>;

/** The view into the information of the guild */
function GuildInfo({ guild, refetch }: IGuildInfoProps) {
    const voice_channels: Array<[ChannelCategory | null, VoiceChannel[]]> = useMemo(() =>
        Array.from(
            guild.voiceChannels
                .reduce<VCMap>((map, x) => {
                    let arr = map.get(x.category);
                    if (arr === undefined) {
                        arr = [];
                    }

                    arr.push(x);

                    map.set(x.category, arr);

                    return map;
                }, new Map())
                .entries()
        )
            .sort(([a], [b]) =>
                (a?.position === undefined ? -1 : a.position) - (b?.position === undefined ? -1 : b?.position)
            ).map(([cat, vcs]) =>
                [cat, vcs.sort((a, b) => a.position - b.position)]
            ),
        [guild.voiceChannels]
    );

    return (
        <div>
            <div>/ <Link to="/">Home</Link> / {guild.name}</div>
            <button onClick={refetch}>Refresh</button>
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
                        <td style={{ color: `#${guild.owner.color === null ? "000000" : guild.owner.color.toString(16)}`, fontWeight: "bold" }}>{guild.owner.nick === null ? guild.owner.name : guild.owner.nick}#{guild.owner.discriminator}</td>
                    </tr>
                    <tr>
                        <td>Voice categories:</td>
                        <td>
                            <table>
                                <thead>
                                    <tr>
                                        <th>Name:</th>
                                        <th>Position:</th>
                                        <th>Voice channels:</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {voice_channels.map(([category, vcs], i) => (
                                        <tr key={i}>
                                            <td>{category?.name === undefined ? "No Category" : category.name}</td>
                                            <td>{category?.position === undefined ? -1 : category.position}</td>
                                            <td>
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
                                                        {vcs.map((vc, j) => (
                                                            <tr key={j}>
                                                                <td><Link to={`/${guild.id}/${vc.id}`}>{vc.name}</Link></td>
                                                                <td>{vc.position}</td>
                                                                <td>{vc.canOperate.toString()}</td>
                                                                <td>{vc.userLimit}</td>
                                                                <td>{vc.states.length}</td>
                                                                <td>{vc.states.map(s => `${s.member.name}#${s.member.discriminator}`).join(", ")}</td>
                                                            </tr>
                                                        ))}

                                                    </tbody>
                                                </table>
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </td>
                    </tr>
                    <tr>
                        <td>Guild icon:</td>
                        <td><img src={getGuildIcon(guild)} alt="Guild icon"/></td>
                    </tr>
                </tbody>
            </table>
        </div >
    );
}