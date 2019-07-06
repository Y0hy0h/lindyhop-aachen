module Pages.Overview exposing
    ( Model
    , init
    , view
    )

import Css exposing (em, inherit, none, px, zero)
import Css.Global as Css
import Events exposing (Event, Location, Locations, Occurrence)
import Html.Styled exposing (Html, a, div, h1, h2, h3, li, ol, p, span, text)
import Html.Styled.Attributes exposing (css, href)
import Http
import IdDict exposing (encodeIdForUrl)
import Routes
import String.Extra as String
import Time
import Utils.NaiveDateTime as Naive
import Utils.TimeFormat as TimeFormat


type alias Model =
    { store : Events.Store
    }


init : Events.Store -> Model
init store =
    { store = store }


view : Model -> List (Html msg)
view model =
    [ h1 [] [ text "Admin" ]
    , h2 [] [ text "Veranstaltungen" ]
    , ol [ css [ listStyle, spreadListItemStyle ] ]
        (Events.mapEvents
            (\id event ->
                li []
                    [ a [ href (Routes.toRelativeUrl <| Routes.EditEvent <| IdDict.encodeIdForUrl id), css [ hiddenLinkStyle ] ]
                        [ viewEvent (Events.locations model.store) event ]
                    ]
            )
            model.store
            ++ [ a [ href (Routes.toRelativeUrl <| Routes.CreateEvent) ] [ text "Neue Veranstaltung" ] ]
        )
    , h2 [] [ text "Orte" ]
    , ol [ css [ listStyle, spreadListItemStyle ] ]
        (Events.mapLocations
            (\id location ->
                li []
                    [ a [ href (Routes.toRelativeUrl <| Routes.EditLocation <| IdDict.encodeIdForUrl id), css [ hiddenLinkStyle ] ]
                        [ viewLocation location ]
                    ]
            )
            model.store
            ++ [ a [ href (Routes.toRelativeUrl <| Routes.CreateLocation) ] [ text "Neuer Ort" ] ]
        )
    ]


hiddenLinkStyle : Css.Style
hiddenLinkStyle =
    Css.batch
        [ Css.color inherit
        , Css.textDecoration inherit
        , Css.hover
            [ Css.color (Css.rgba 0 0 0 0.6)
            ]
        ]


listStyle : Css.Style
listStyle =
    Css.batch
        [ Css.listStyleType none
        , Css.padding zero
        ]


spreadListItemStyle : Css.Style
spreadListItemStyle =
    Css.batch
        [ Css.children
            [ Css.typeSelector "li"
                [ Css.adjacentSiblings
                    [ Css.typeSelector
                        "li"
                        [ Css.marginTop (em 0.5)
                        ]
                    ]
                ]
            ]
        ]


viewEvent : Locations -> Event -> Html msg
viewEvent locations event =
    let
        max =
            5

        occurrencesPreview =
            List.take max event.occurrences

        remaining =
            List.length event.occurrences - List.length occurrencesPreview

        occurrenceListItems =
            List.map (\occurrence -> li [] [ viewOccurrence locations occurrence ]) occurrencesPreview

        hintStyle =
            Css.fontStyle Css.italic

        listItems =
            if List.length occurrenceListItems == 0 then
                [ span [ css [ hintStyle ] ] [ text "Keine Termine." ] ]

            else
                occurrenceListItems
                    ++ (if remaining > 0 then
                            let
                                pluralized =
                                    if remaining > 1 then
                                        "weitere Termine"

                                    else
                                        "weiterer Termin"
                            in
                            [ li [ css [ hintStyle ] ] [ text <| "(+" ++ String.fromInt remaining ++ " " ++ pluralized ++ ")" ] ]

                        else
                            []
                       )

        shortenedDescription =
            String.softEllipsis 100 event.description
    in
    div
        [ css
            [ Css.property "display" "grid"
            , Css.property "grid-template-columns" "repeat(auto-fit, minmax(6em, 1fr))"
            , itemBoxStyle
            ]
        ]
        [ div []
            [ h3 [ css [itemHeadingStyle ] ] [ text event.title ]
            , p [] [ text event.teaser ]
            , p [] [ text shortenedDescription ]
            ]
        , ol [ css [ listStyle, Css.paddingLeft (em 1) ] ] listItems
        ]


viewOccurrence : Locations -> Occurrence -> Html msg
viewOccurrence locations occurrence =
    let
        location =
            IdDict.get occurrence.locationId locations
    in
    div []
        [ text <| TimeFormat.fullDate occurrence.start ++ " - " ++ location.name ]


viewLocation : Location -> Html msg
viewLocation location =
    div [css[itemBoxStyle]]
        [ h3 [css[itemHeadingStyle]] [text <| location.name]
        , p [] [text location.address]
        ]


itemBoxStyle : Css.Style
itemBoxStyle =
    Css.batch
        [ Css.border3 (px 1) Css.solid (Css.rgb 0 0 0)
        , Css.padding (em 1)
        ]

itemHeadingStyle : Css.Style
itemHeadingStyle =
    Css.batch [
 Css.margin zero, Css.marginBottom (em 1)
    ]