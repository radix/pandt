module Elements exposing (..)

-- A module of higher level abstractions for UI elements.

import Html exposing (..)
import Html.Attributes exposing (..)

hbox : List (Html a) -> Html a
hbox els = div [style [("display", "flex"), ("width", "100%")] ]
               (List.map (\el -> div [style [("flex-grow", "1")]] [el]) els)

vbox : List (Html a) -> Html a
vbox els = div [style [("display", "flex"), ("flex-direction", "column"), ("width", "100%")]]
               (List.map (\el -> div [style [("flex-grow", "1")]] [el]) els)