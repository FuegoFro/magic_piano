\version "2.23.6"
\language "english"

tenorMusic = \relative {
  \partial 2. c'4 d f |
  g1~ |
  4 a8 a g( f) d4 |
  f1~ |
  4 c8 8 d4 f |
  g1~ |
  4 f8 g f4 f |
  f1~ |
  4 r r2 |
}
leadMusic = \relative {
  \partial 2. c'4 c c |
  d1~ |
  4 d8 d d4 bf |
  a1~ |
  4 c8 8 c4 c |
  d2( cs |
  c!4) c8 c b4 bf |
  a1~ |
  4 r r2 |
}
bariMusic = \relative {
  \partial 2. c'4 c a |
  f1~ |
  4 f8 f f4 f |
  c1~ |
  4 c'8 8 c4 a |
  f2( e |
  ef4) ef8 ef d4 df |
  c1~ |
  4 r r2 |
}
bassMusic = \relative {
  \partial 2. c'4 a f |
  bf,1~ |
  4 bf8 bf bf4 bf |
  f1~ |
  4 c''8 8 a4 f |
  bf,2( a |
  af4) af8 af g4 gf |
  f1~ |
  4 r r2 |
}

songLyrics = \lyricmode {
  And when I die __ you can bur -- y me __
  'neat the west -- ern sky, __ __ __ on the lone prai -- rie. __
}

\book {
  \header {
    title = "Lone Prairie"
    composer = "Normal Luboff"
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
        \key f \major
        <<
          \new Voice = "tenor" {
            \set midiInstrument = #"tenor"
            \voiceOne
%            \override NoteColumn.force-hshift = #1.7
            \tenorMusic
          }
          \new Voice = "lead" {
            <<
            \set midiInstrument = #"lead"
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
        \key f \major
        <<
          \new Voice = "bari" {
            \set midiInstrument = #"bari"
            \voiceOne
%            \override NoteColumn.force-hshift = #1.7
            \bariMusic

          }
          \new Voice = "bass" {
            \set midiInstrument = #"bass"
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
        \Score midiChannelMapping = #'instrument
      }
    }
  } % score
} % book
