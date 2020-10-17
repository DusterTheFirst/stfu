import { DISCORD_CDN } from "./constants";

type SimpleMember = {
    id: string,
    avatar: string | null,
    discriminator: string
};

/** Get the avatar for a user given their  */
export function getAvatar(member: SimpleMember): string {
    if (member.avatar === null) {
        return `${DISCORD_CDN}embed/avatars/${parseInt(member.discriminator, 10) % 5}.png`;
    } else {
        return `${DISCORD_CDN}avatars/${member.id}/${member.avatar}.${member.avatar?.startsWith("a_") ? "gif" : "webp"}`;
    }
}

type SimpleGuild = {
    id: string,
    icon: string | null,
}

/** Get the icon for a guild */
export function getGuildIcon(guild: SimpleGuild): string | undefined {
    if (guild.icon === null) {
        return undefined;
    } else {
        return `${DISCORD_CDN}icons/${guild.id}/${guild.icon}.${guild.icon.startsWith("a_") ? "gif" : "webp"}`;
    }
}