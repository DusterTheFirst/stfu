/* tslint:disable */
/* eslint-disable */
// @generated
// This file was automatically generated and should not be edited.

// ====================================================
// GraphQL query operation: Me
// ====================================================

export interface Me_me {
  __typename: "Me";
  name: string;
  id: string;
  discriminator: string;
}

export interface Me {
  /**
   * Get information about the bot user
   */
  me: Me_me | null;
}
