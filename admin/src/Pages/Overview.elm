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
    [ h1 [ css [ Css.marginBottom zero ] ] [ text "Lindy Hop Aachen Admin" ]
    , a [ href "/" ] [ text "Zurück zur Website" ]
    , h2 [] [ text "Veranstaltungen" ]
    , a [ href (Routes.toRelativeUrl <| Routes.CreateEvent) ] [ text "Neue Veranstaltung" ]
    , ol [ css [ listStyle, spreadListItemStyle, Css.marginTop (em 0.5) ] ]
        (Events.mapEvents
            (\id event ->
                li []
                    [ a [ href (Routes.toRelativeUrl <| Routes.EditEvent <| IdDict.encodeIdForUrl id), css [ hiddenLinkStyle, focusBoxStyle ] ]
                        [ viewEvent (Events.locations model.store) event ]
                    ]
            )
            model.store
        )
    , h2 [] [ text "Orte" ]
    , a [ href (Routes.toRelativeUrl <| Routes.CreateLocation) ] [ text "Neuer Ort" ]
    , ol [ css [ listStyle, spreadListItemStyle, Css.marginTop (em 0.5) ] ]
        (Events.mapLocations
            (\id location ->
                li []
                    [ a [ href (Routes.toRelativeUrl <| Routes.EditLocation <| IdDict.encodeIdForUrl id), css [ hiddenLinkStyle, focusBoxStyle ] ]
                        [ viewLocation location ]
                    ]
            )
            model.store
        )
    ]


hiddenLinkStyle : Css.Style
hiddenLinkStyle =
    Css.batch
        [ Css.color inherit
        , Css.textDecoration inherit
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
            , Css.property "grid-template-columns" "repeat(auto-fit, minmax(12em, 1fr))"
            , itemBoxStyle
            ]
        ]
        [ div []
            [ h3 [ css [ itemHeadingStyle ] ] [ text event.title ]
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
    div [ css [ itemBoxStyle ] ]
        [ h3 [ css [ itemHeadingStyle ] ] [ text <| location.name ]
        , p [] [ text location.address ]
        ]


itemBoxStyle : Css.Style
itemBoxStyle =
    Css.batch
        [ Css.boxSizing Css.borderBox
        , Css.padding (em 1)
        ]


focusBoxStyle : Css.Style
focusBoxStyle =
    let
        focusStyle =
            Css.batch
                [ Css.outlineWidth (px 3)
                ]
    in
    Css.batch
        [ Css.outline3 (px 1) Css.solid (Css.rgb 0 0 0)
        , Css.hover [ focusStyle ]
        , Css.focus [ focusStyle ]
        ]


itemHeadingStyle : Css.Style
itemHeadingStyle =
    Css.batch
        [ Css.margin zero
        , Css.marginBottom (em 1)
        ]
