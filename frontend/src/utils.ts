import { DISCORD_CDN } from "./constants";

/** A stripped down, barebones member for the gatAvatar function */
interface ISimpleMember {
    /** The user id */
    id: string;
    /** The user avatar */
    avatar: string | null;
    /** The user discriminator */
    discriminator: string;
}

/** Get the avatar for a user given their  */
export function getAvatar(member: ISimpleMember, should_animate = true): string {
    if (member.avatar === null) {
        return `${DISCORD_CDN}embed/avatars/${parseInt(member.discriminator, 10) % 5}.png`;
    } else {
        return `${DISCORD_CDN}avatars/${member.id}/${member.avatar}.${member.avatar?.startsWith("a_") && should_animate ? "gif" : "webp"}`;
    }
}

/** A stripped down, bare bones guild for the getGuildIcon function */
interface ISimpleGuild {
    /** The id of the guild */
    id: string;
    /** The icon of the guild */
    icon: string | null;
}

/** Get the icon for a guild */
export function getGuildIcon(guild: ISimpleGuild, should_animate = true): string | undefined {
    if (guild.icon === null) {
        return undefined;
    } else {
        return `${DISCORD_CDN}icons/${guild.id}/${guild.icon}.${guild.icon.startsWith("a_") && should_animate ? "gif" : "webp"}`;
    }
}