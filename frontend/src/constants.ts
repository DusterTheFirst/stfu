/** Ia true if the app is in production mode */
export const IS_PROD = process.env.NODE_ENV === "production";

/** Is true if the backend uses https */
export const IS_HTTPS = IS_PROD;
/** The backend domain */
export const BACKEND_DOMAIN = IS_PROD ? "stfu-backend.dusterthefirst.com" : "dev.stfu-backend.dtf.com:8000";
/** The backend graphql url */
export const BACKEND_GRAPHQL_URL = `http${IS_HTTPS ? "s" : ""}://${BACKEND_DOMAIN}/graphql`;

/** The CDN for accessing images or content from discord */
export const DISCORD_CDN = "https://cdn.discordapp.com/";
