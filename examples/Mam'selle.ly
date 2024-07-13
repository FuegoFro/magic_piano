\version "2.23.6"
\language "english"

tenorMusic = \relative {
  \partial 2. d4 e g |
  e'4. c8 4. 8 |
  g'4. 8 4 r |
  r e8 d c4 d |
  ef2. 4 |
  g2 2 |
  2 8([ ef] f fs) |
  g1 |
}
leadMusic = \relative {
  \partial 2. d4 e g |
  c4. g8 fs4. e8 |
  e'4. c8 a4 fs |
  g2. 4 |
  a2. 4 |
  d2 2 |
  cs c |
  b1 |
}
bariMusic = \relative {
  \partial 2. d4 e e |
  g4. e8 d4. g8 |
  a4. a8 c4 r |
  r a8 fs e4 e |
  c2. 4 |
  f2 2 |
  e2 ef8( f ef4) |
  d1 |
}
bassMusic = \relative {
  \partial 2. d4 e e |
  a,4. a8 a4. a8 |
  c4. e8 ef4 r |
  r c8 b a4 b |
  f2. 4 |
  bf2 2 |
  a2 af |
  g1 |
}

songLyrics = \lyricmode {
  Then vio -- o -- lins will cry, and so will I,
  mam' -- selle __ __ __ will cry, __ and so __ will__ I,__
  mam' -- -- -- -- selle. __
}

extraLyrics = \lyricmode {
  _ _ _
  _ _ _ _
  _ _ _
  vi -- o -- lins
}



\book {
  \header {
    title = "Mam'selle"
%    composer = ""
    tagline = ##f
  }
  \score {
    \new ChoirStaff
    <<
      \new Lyrics = "extraLyrics1"
      \new Staff \with {
        instrumentName = \markup { \column { Tenor Lead } }
        \consists Merge_rests_engraver
      } {
        \clef "treble_8"
        \key g \major
        <<
          \new Voice = "tenor" {
            \voiceOne
%            \override NoteColumn.force-hshift = #1.7
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
      \new Lyrics = "extraLyrics2"
      \new Staff \with {
        instrumentName = \markup { \column { Bari. Bass } }
        \consists Merge_rests_engraver
      } {
        \clef "bass"
        \key g \major
        <<
          \new Voice = "bari" {
            \voiceOne
%            \override NoteColumn.force-hshift = #1.7
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

      \context Lyrics = "extraLyrics1" {
        \lyricsto "tenor" {
          \extraLyrics
        }
      }

      \context Lyrics = "extraLyrics2" {
        \lyricsto "bari" {
          \extraLyrics
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
