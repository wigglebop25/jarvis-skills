# JARVIS Workspace Protocols

## Spotify Protocol (STRICT ENFORCEMENT)

For ANY Spotify-related request (play, search, pause, skip, etc.), you MUST follow this sequence:

1.  **Check Auth First**: ALWAYS call `checkSpotifyAuth` FIRST before responding or asking questions.
2.  **Handle Unauthenticated**: If `authenticated` is `false`:
    *   IMMEDIATELY call `authorizeSpotify` to trigger the auto-open browser flow.
    *   Inform the user that the browser has been opened for them to login.
    *   **STOP**. Do NOT ask what to play or search until they are logged in.
3.  **Handle Authenticated**: Only if `authenticated` is `true`, proceed to call the requested tool (`searchSpotify`, `playMusic`, etc.) or ask for clarification if needed.

**Goal**: Never ask the user "What would you like to play?" if they aren't even logged in yet.
