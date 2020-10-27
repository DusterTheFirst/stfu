import { gql, useQuery } from "@apollo/client";
import React from "react";
import { Link } from "react-router-dom";
import ErrorScreen from "../components/Error";
import { LoadingIcon, LoadingScreen } from "../components/Loading";
import { getGuildIcon } from "../utils";
import { GetSharedGuilds, GetSharedGuilds_sharedGuilds } from "./__generated__/GetSharedGuilds";

/** The graphql query to get information about the shared guilds between the user and bot */
const GET_SHARED_GUILDS = gql`
    query GetSharedGuilds {
        sharedGuilds {
            icon
            id
            name
            owner {
                id
                discriminator
                name
                nick
                color
            }
            voiceChannels {
                canOperate
            }
            me {
                id
                discriminator
                name
                nick
                color
            }
        }
    }
`;

/** The page for guild information */
export default function Overview() {
    const { loading, error, data, refetch } = useQuery<GetSharedGuilds>(GET_SHARED_GUILDS, { notifyOnNetworkStatusChange: true });
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
        const guilds = data!.sharedGuilds;

        return (
            <>
                {loading ? <LoadingIcon /> : undefined}
                <GuildsInfo guilds={guilds} refetch={refetch_no_await} />
                <pre>
                    {JSON.stringify(data, undefined, 4)}
                </pre>
            </>
        );
    }
}

/** The props for the GuildInfo component */
interface IGuildsInfoProps {
    /** The guilds to view */
    guilds: Readonly<GetSharedGuilds_sharedGuilds[]>;
    /** The refresh function to query a refresh */
    refetch(): void;
}

/** The view into the information of the guild */
function GuildsInfo({ guilds, refetch }: IGuildsInfoProps) {
    return (
        <div>
            <div>/ Home</div>
            <button onClick={refetch}>Refresh</button>
            <table>
                <thead>
                    <tr>
                        <th>Guild Name</th>
                        <th>Guild Icon</th>
                        <th>Guild Id</th>
                        <th>Guild Owner</th>
                        <th>You</th>
                        <th>Operable</th>
                    </tr>
                </thead>
                <tbody>
                    {[...guilds].sort((a, b) => a.name.localeCompare(b.name)).map((guild) => (
                        <tr key={guild.id}>
                            <td><Link to={`/${guild.id}`}>{guild.name}</Link></td>
                            <td><img src={getGuildIcon(guild)} alt="Guild icon" /></td>
                            <td>{guild.id}</td>
                            <td style={{ color: `#${guild.owner.color === null ? "000000" : guild.owner.color.toString(16)}`, fontWeight: "bold" }}>{guild.owner.nick === null ? guild.owner.name : guild.owner.nick}#{guild.owner.discriminator}</td>
                            <td style={{ color: `#${guild.me.color === null ? "000000" : guild.me.color.toString(16)}`, fontWeight: "bold" }}>{guild.me.nick === null ? guild.me.name : guild.me.nick}#{guild.me.discriminator}</td>
                            <td>{guild.voiceChannels.some(x => x.canOperate).toString()}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div >
    );
}
