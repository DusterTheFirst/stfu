import React from "react";

/** The props for the loading screen */
interface ILoadingScreenProps {
    /** A way to refresh the query */
    refetch(): void;
}

/** A loading screen component */
export function LoadingScreen({ refetch }: ILoadingScreenProps) {
    return (
        <div>
            <h1>Loading...</h1>
            <button onClick={refetch}>Retry</button>
        </div>
    );
}

/** A standalone, unobtrusive loading icon */
export function LoadingIcon() {
    // TODO:
    return (<b>Loading...</b>);
}