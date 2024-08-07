/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        // Taken from https://stackoverflow.com/a/77701985/3000133
        files: ["./src/**/*.rs"],
        extract: {
            // Support for leptos class:classname=predicate and
            // class=("classname", predicate) syntax.
            // Without this the tuple syntax works but not
            // the inline syntax.
            rs: (content) => content
                .match(/(?<=class[:=]\(?"?)[-\w: ]+/g)
                ?.flatMap(s => s.split(' ')) ?? []

        }
    },
    theme: {
        extend: {
            colors: {
                noteCursor: "#99ef97",
                startCursor: "#81f7fd",
            },
        },
    },
    plugins: [],
}
