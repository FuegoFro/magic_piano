# Problem
I enjoy using the piano to learn music, I think because it gives me a lot of control over when to play each note. I can play it slower or faster, play just the bits I want to hear again, go back and forth between two notes to work on an interval, play a single note as I'm going to spot check if I'm in tune, etc. However I'm quite bad at piano and can only play one line/voice at a time, and even then I'm much slower at it when it's in bass clef. There are times when I'd like to have the same control over and freedom to mess around with the timing of the notes, but also want to hear the other parts, or sometimes *only* want to hear the other parts but am not talented enough to do so.

# Ideas
1. Practice and learn to play the piano well enough to be able to play all the parts at real-time while also singing.
2. Build an application that allows me to step through and jump around in the playback of sheet music very simply.

# App brainstorming
- Maybe a single button you press repeatedly? It's certainly simple and allows advancing at a custom rate, but it doesn't allow going back easily. You could do things like set your start point (eg with a mouse) and then have another button to reset to the start point, maybe also a button to go back one note or to the beginning of the measure. I think that would be okay, but it's not the same as just jumping to the part you want to play. On the piano each button press creates the sound I want to hear and I think that's important, but maybe this could be an alternative mode.
- Maybe assign each sequential key on the keyboard as a spot in the song, again with a customizable starting point. Could do it fully one-handed, so qwerasdfzxcv or something. Could have a special marker in the music to reset/advance to the next section, eg at the end of a phrase, which would allow you to "play" through the song using just the one hand. That marker would be it's own keypress that would be silent but needed to advance the song/reasign the keys.
- Click on the score to set a start point, or maybe to select a section depending on what above looks like. Maybe option for either (eg shift click to force a start point, dunno how that handles sections).
- Interaction points would be note changes. Maybe customizable whether it's note changes in your part or all parts (could see wanting both).
- Full mixer control over the parts, independent of choosing interaction points above. Eg I want to be able to hear just my part, all the parts, everything except my part (normal mute/solo controls), but also adjust relative volumes would be nice.
- Interactions always rearticulate held notes? Maybe not needed if we can make them hold properly.
- Would love a full visual render of the music, highlighting the notes that were just played as well as a line/arrow where we are.
- Would be very cool to be able to share a given piece, eg via a link, but that starts getting into some spooky DMCA territory. Maybe export a file that people figure out how to share with others on their own. Could store in local storage to avoid needing to reupload each time.
- Data format is maybe... something like lilypond? Ideally some existing, open source thing. It does look like that can be converted into MIDI. Maybe just MIDI then? What is MusicXML and would it be useful here?
- Thinking of a web app, maybe fully in Rust?

# Todo
- [x] Use midly and midir to parse a MIDI and play it one interaction point at a time.
- [ ] Volume/mute/solo controls for each voice
- [x] Better loading, loading indicators
- [x] Handle race where notes get stuck "on"
- [ ] Work from MusicXML
- [ ] Upload music files, not hard coded
- [ ] Actually render the sheet music
  - [ ] Show current (most recent) position
  - [ ] Grey out notes for muted voices
  - [ ] Render key to press above each note slice
- [ ] Various web accoutrements (favicon, title, ...metadata?)
