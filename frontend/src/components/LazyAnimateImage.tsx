import React, { useState } from "react";

/** the props for the `LazyAnimateImage` components */
interface IProps extends React.DetailedHTMLProps<React.ImgHTMLAttributes<HTMLImageElement>, HTMLImageElement> {
    /** The callback to compute the source of the image */
    source(hover: boolean): string;
}

/** A helper component to only animate images on hover */
export default function LazyAnimateImage({ source, ...rest }: IProps) {
    const [hover, setHover] = useState(false);

    return <img {...rest} alt={rest.alt} src={source(hover)} onMouseEnter={setHover.bind(setHover, true)} onMouseLeave={setHover.bind(setHover, false)} />;
}