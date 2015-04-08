// Languages: name (local), name_en, name_fr, name_es, name_de
@name: '[name_en]';

// Common Colors //
@water: hsl(240, 100%, 0%);

@road:  #aaa;
@land:  #111111;

@fill1: #000000;
@fill2: #444444;
@fill3: #aaa;
@fill4: #ffffff;
@fill5: #7a7a7a;
@fill6: #ccc;

@text: #bbb;


@fallback: 'Open Sans Regular';
@sans: 'Open Sans Regular', 'Arial Unicode MS Regular';
@sans_md: 'Open Sans Semibold', 'Arial Unicode MS Regular';
@sans_bd: 'Open Sans Bold','Arial Unicode MS Bold';
@sans_it: 'Open Sans Italic', 'Arial Unicode MS Regular';
@sans_bdit: 'Open Sans Bold Italic','Arial Unicode MS Bold';

/*@sans: 'Meta Offc Pro Cond Normal', 'Arial Unicode MS Regular';
@sans_it: 'Meta Offc Pro Light Italic', 'Arial Unicode MS Regular';
@serif_md: 'Meta Serif SC Offc Pro Medium', 'Arial Unicode MS Regular';*/


#water {
  opacity: 1.0;
  polygon-fill: @water;
}


#mapbox_satellite_full,
#mapbox_satellite_watermask  {
  raster-opacity: 1;
  
  image-filters: gray(), scale-hsla(0, 1, 0, 0, 0, 0.7, 0, 1);
  //image-filters: gray();
}
#mapbox_satellite

// Southern Hemisphere:
#hillshade {
  comp-op: overlay;
  polygon-opacity: 0.0;
  [class='medium_shadow'] { polygon-fill: #46a; }
  [class='full_shadow'] { polygon-fill: #246; }
  [class='medium_highlight'] { polygon-fill: #ea8; }
  [class='full_highlight'] { polygon-fill: #fea; }
}

#contour::line[index!=-1] {
  line-color: @fill3;
  line-opacity: 0.15;
  line-width: 1;
  [index>=5] {
    line-opacity: 0.3;
    line-width: 1.2;
  }
}
#contour::label {
  [zoom >= 10][index = 10], [zoom >= 15][index = 5] {
    text-opacity: 0.8;
    text-name: [ele];
    text-fill: @text;
    text-face-name: 'Open Sans Regular';
    text-size: 10;
    [zoom >= 17] { text-size: 14; }
    text-placement: line;
    text-halo-fill: black;
    text-halo-opacity: 0.3;
    text-halo-radius: 2;
  }
}


#building {
  polygon-fill: white;
  opacity: 0.05;
}

#admin[admin_level=2][zoom>=4] {
  [maritime=0] {
    ::case {
      opacity: 0.5;
      line-color: @water;
      line-join: round;
      line-cap: round;
      
      line-width: 1;
      [zoom>=3] { line-width: 3; }
      [zoom>=6] { line-width: 5; }
    }
    ::fill {
      line-color: @fill6;
      line-join: round;
      line-cap: round;
      line-width: 0.4;
      [zoom>=3] { line-width: 0.6; }
      [zoom>=6] { line-width: 1; }
    }
  }
  [maritime=1] { line-color: #555; line-dasharray: 3,2; }
}
#admin[admin_level=4][maritime=0][zoom>=6] {
  ::case {
    line-color: @water; 
    line-join: round;
    line-cap: round;
    line-width: 3;
    [zoom=6] { line-opacity: 0.4;}
    [zoom=7] { line-opacity: 0.5; }
    [zoom>=8] { line-opacity: 1.0; }

  }
  ::fill {
    line-color: @fill6;
    line-join: round;
    line-cap: round;
    line-width: 0.6;
    line-dasharray: 2,4;
    [zoom=6] { line-opacity: 0.7;}

  }
}
  
#waterway[type="river"][zoom>=8][zoom<10] {
  line-color: black;
  line-width: 0.5;
  line-opacity: 0.5;
}
#waterway[zoom>=10] {
  line-width: 0;
  [type="river"] { line-width: 3; }
  [type="stream"],
  [type="canal"] { line-width: 1; }
  [class="stream_intermittent"] { line-dasharray: 2, 4; }
  line-opacity: 0.6;
  line-color: black;
}
