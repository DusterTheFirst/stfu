import { createGlobalStyle } from "styled-components";

/** The global style for the application */
export const GlobalStyle = createGlobalStyle`
    body {
        margin: 0;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
            'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
            sans-serif;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
    }

    code {
        font-family: source-code-pro, Menlo, Monaco, Consolas, 'Courier New',
            monospace;
    }

    table {
        border: 2px solid black;
        width: 100%;

        td {
            border: 2px solid black;
            text-align: center;
        }
        th {
            background: black;
            color: white;
            text-align: center;
        }
    }
`;