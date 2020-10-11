/* tslint:disable */
/* eslint-disable */
// @generated
// This file was automatically generated and should not be edited.

// ====================================================
// GraphQL query operation: BotMeta
// ====================================================

export interface BotMeta_me {
  __typename: "Me";
  name: string;
  id: string;
  discriminator: string;
}

export interface BotMeta {
  /**
   * Get information about the bot user
   */
  me: BotMeta_me | null;
}
