import { ApolloError, ServerError } from "@apollo/client";
import React from "react";
import { BACKEND_DOMAIN } from "../constants";

/** The props sent to the error screen */
interface IErrorScreenProps {
    /** Function to trigger a refetch */
    refetch(): void;
    /** The error that caused this screen */
    error: ApolloError;
}

/** A full screen error */
export default function ErrorScreen({ refetch, error }: IErrorScreenProps) {
    if (error.networkError !== null && (error.networkError as Partial<ServerError>).statusCode === 401) {
        return (
            <div>
                <h1>Unauthenticated</h1>
                <p>You have to log in to access this page</p>
                <a href={`//${BACKEND_DOMAIN}/oauth/login?from=${encodeURIComponent(window.location.href)}`}>Login</a>
            </div>
        );
    } else {
        return (
            <div>
                <h1>Error :(</h1>
                <button onClick={refetch}>Retry</button>
                <pre>{JSON.stringify(error, undefined, 4)}</pre>
            </div>
        );
    }
}
