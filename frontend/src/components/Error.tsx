import { ApolloError } from "@apollo/client";
import React from "react";

/** The props sent to the error screen */
interface IErrorScreenProps {
    /** Function to trigger a refetch */
    refetch(): void;
    /** The error that caused this screen */
    error: ApolloError;
}

/** A full screen error */
export default function ErrorScreen({refetch, error}: IErrorScreenProps) {
    // if (error.networkError?.message = ) {
    //     return (
    //         <div>
    //             <h1>Network Error :(</h1>
    //             <p>Make sure you are connected to the internet</p>
    //             <p>If the issue persists, the backend may be offline</p>
    //             <button onClick={refetch_no_await}>Retry</button>
    //         </div>
    //     );
    // } else {
    return (
        <div>
            <h1>Error :(</h1>
            <button onClick={refetch}>Retry</button>
            <pre>{JSON.stringify(error, undefined, 4)}</pre>
        </div>
    );
    // }
}