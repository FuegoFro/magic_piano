\version "2.23.6"
\language "english"

tenorMusic = \relative {
  \partial 2 s2 |
  gf'2 f |
  ef ef4 df |
  df2 r |
  bf bf |
  df~ 4. 8 |
  cf2~ 2( |
  bf) r2 |
}
leadMusic = \relative {
  \partial 2 gf4 af |
  df2 gf, |
  df' cf4 4 |
  bf2 gf4 af |
  df2 gf, |
  gf2( f4.) f8 |
  gf2~ 2( |
  2) r2 |
}
bariMusic = \relative {
  \partial 2 r2 |
  bf2 df |
  gf, gf4 af |
  af2 r |
  af ef |
  af~ 4. 8 |
  ef2( bf')( |
  df,) r |
}
bassMusic = \relative {
  \partial 2 r2 |
  gf,2 bf |
  cf cf4 gf' |
  gf2 r |
  gf, bf |
  df~ 4. 8 |
  cf2( ef)( |
  gf,) r |
}

songLyrics = \lyricmode {
  There's a mill -- ion stars in the skies,
  none shine brigh -- ter than __ your eyes __
}



\book {
  \header {
    title = "A Million Stars"
    composer = "Jake Kynaston"
    tagline = ##f
  }
  \score {
    \new ChoirStaff
    <<
      \new Staff \with {
        instrumentName = \markup { \column { Tenor Lead } }
        \consists Merge_rests_engraver
      } {
        \clef "treble_8"
        \key gf \major
        <<
          \new Voice = "tenor" {
            \voiceOne
            \override NoteColumn.force-hshift = #1.7
            \tenorMusic
          }
          \new Voice = "lead" {
            <<
            \voiceTwo
            \leadMusic
            >>
          }
        >>
      }
      \new Lyrics = "lyrics"
      \new Staff \with {
        instrumentName = \markup { \column { Bari. Bass } }
        \consists Merge_rests_engraver
      } {
        \clef "bass"
        \key gf \major
        <<
          \new Voice = "bari" {
            \voiceOne
            \override NoteColumn.force-hshift = #1.7
            \bariMusic

          }
          \new Voice = "bass" {
            \voiceTwo
            \bassMusic
          }
        >>
      }

      \context Lyrics = "lyrics" {
        \lyricsto "lead" {
          \songLyrics
        }
      }
    >>

    \layout { }
    \midi {
      \tempo 4 = 120
      \context {
        \Score midiChannelMapping = #'voice
      }
    }
  } % score
} % book
