// dummy styles to set ordering (case below fill)
#tunnel, #bridge, #road {
  ::case { opacity: 1; }
  ::fill { opacity: 1; }
} 

// consistent case size
@case: 2;

// Road & Railway Fills //
#tunnel { opacity: 0.5; }


#road[zoom>=5],
#tunnel[zoom>=5],
#bridge[zoom>=5] {
  line-color: @road;
  opacity: 1;
  line-width: 0.3;
  [zoom <= 9] { opacity: 0.9; }
  [zoom = 6] { line-width: 0.2; }
  [class='motorway'] { line-width: 0.5;}
  [zoom = 5] { line-width: 0.2; }
}

#road::fill[zoom>=10],
#tunnel::fill[zoom>=10],
#bridge::fill[zoom>=10] {
  ['mapnik::geometry_type'=2] {
    line-color: @road;
    opacity: 1;
    [zoom=10] { opacity: 0.5; }
    line-width: 0.5;
    [zoom>=15] { line-width: 1; } 
    [class='path'] { line-dasharray: 2,2;}
    [class='major_rail'],
    [class='minor_rail'] { line-dasharray: 3,3; }
    [class='motorway'] { 
      [zoom>=10] { line-width: 2; }
      [zoom>=12] { line-width: 3; }
      [zoom>=14] { line-width: 4; }
      [zoom>=16] { line-width: 7; }
      [zoom>=18] { line-width: 10; }
    }
    [class='motorway_link'],
    [class='main'] {
      [zoom>=10] { line-width: 1; }
      [zoom>=12] { line-width: 2; }
      [zoom>=14] { line-width: 3; }
      [zoom>=16] { line-width: 5; }
      [zoom>=18] { line-width: 7; }
    }
    [class='street'],
    [class='street_limited'] {
      [zoom>=14] { line-width: 1; }
      [zoom>=16] { line-width: 2; }
      [zoom>=18] { line-width: 4; }
    }
    [class='street_limited'] { line-dasharray: 4,2; }
  }
}

// Casing for high-zoom roads //
#road::case[zoom>=9],
#tunnel::case[zoom>=9],
#bridge::case[zoom>=9] {
  ['mapnik::geometry_type'=2][class='motorway'] {

    line-color: black;
    line-width: @case;
    [zoom>=6][zoom < 10] { line-width: 0.5 + @case; }
    [zoom>=10] { line-width: 2 + @case; }
    [zoom>=12] { line-width: 3 + @case; }
    [zoom>=14] { line-width: 4 + @case; }
    [zoom>=16] { line-width: 7 + @case; }
    [zoom>=18] { line-width: 10 + @case; }
  }
}
#road::case[zoom>=11],
#tunnel::case[zoom>=11],
#bridge::case[zoom>=11] {
  ['mapnik::geometry_type'=2] {
    line-color: black;
    [zoom = 11] { line-color: #333; }
    line-width: @case;
    [class='motorway_link'] { 
      [zoom>=8] { line-width: 1 + @case; }
      [zoom>=11] { line-width: 2 + @case; }
      [zoom>=12] { line-width: 3 + @case; }
      [zoom>=14] { line-width: 4 + @case; }
      [zoom>=16] { line-width: 7 + @case; }
      [zoom>=18] { line-width: 10 + @case; }
    }
    [class='motorway_link'],
    [class='main'] {
      [zoom>=11] { line-width: 1 + @case; }
      [zoom>=12] { line-width: 2 + @case; }
      [zoom>=14] { line-width: 3 + @case; }
      [zoom>=16] { line-width: 5 + @case; }
      [zoom>=18] { line-width: 7 + @case; }
    }
    [class='street'],
    [class='street_limited'] {
      [zoom>=14] { line-width: 1 + @case; }
      [zoom>=16] { line-width: 2 + @case; }
      [zoom>=18] { line-width: 4 + @case; }
    }
    [class='street_limited'] { line-dasharray: 4,2; }
  }
}