module Pages.Overview exposing
    ( LoadModel
    , LoadMsg
    , Model
    , init
    , updateLoad
    , view
    )

import Css exposing (em, inherit, none, zero)
import Css.Global as Css
import Events exposing (Event, Location, Locations, Occurrence)
import Html.Styled exposing (Html, a, div, h1, h2, li, ol, text)
import Html.Styled.Attributes exposing (css, href)
import Http
import IdDict exposing (encodeIdForUrl)
import Routes
import Time
import Utils.TimeFormat as TimeFormat


type alias Model =
    { store : Events.Store
    }


type alias LoadModel =
    {}


init : ( LoadModel, Cmd LoadMsg )
init =
    let
        fetchEvents =
            Events.fetchEvents FetchedEvents
    in
    ( LoadModel, fetchEvents )


type LoadMsg
    = FetchedEvents (Result Http.Error Events.Store)


updateLoad : LoadMsg -> LoadModel -> Result Http.Error Model
updateLoad msg model =
    case msg of
        FetchedEvents result ->
            Result.map Model result


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

        doesOverflow =
            List.length event.occurrences > max

        occurrenceListItems =
            List.map (\occurrence -> li [] [ viewOccurrence locations occurrence ]) occurrencesPreview

        listItems =
            occurrenceListItems
                ++ (if doesOverflow then
                        [ li [] [ text "…" ] ]

                    else
                        []
                   )
    in
    div []
        [ text event.name
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
    div []
        [ text <| location.name ++ " (" ++ location.address ++ ")"
        ]
