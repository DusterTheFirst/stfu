const { cwd } = require("process")

// Fix to get around the problem of different directories
const localSchemaFile = cwd().endsWith("frontend") ? '../target/graphql.json' : 'target/graphql.json';

module.exports = {
    client: {
        service: {
            name: "stfu",
            localSchemaFile,
            url: "http://localhost:8000/graphql"
        }
    }
}