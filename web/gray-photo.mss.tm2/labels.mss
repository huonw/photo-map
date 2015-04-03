
#country_label[zoom>=5], #country_label[zoom=4][scalerank<=3] {
  text-name: @name;
  text-face-name: @sans_md;
  text-fill: @fill6;
  text-halo-fill: fadeout(darken(@water,10),95);
  text-halo-radius: 3;
  text-halo-rasterizer: fast;
  text-wrap-width: 60;
  [zoom>=3] {
    text-size: 10;
    [scalerank<=6] { text-size: 11; }
    [scalerank<=4] { text-size: 12; }
    [scalerank<=2] { text-size: 13; }
  }
  [zoom>=5] {
    text-size: 11;
    [scalerank<=6] { text-size: 13; }
    [scalerank<=4] { text-size: 15; }
    [scalerank<=2] { text-size: 17; }
  }
  [zoom>=6] {
    text-size: 14;
    [scalerank<=6] { text-size: 16; }
    [scalerank<=4] { text-size: 18; }
    [scalerank<=2] { text-size: 20; }
  }
}


// ---------------------------------------------------------------------
// Cities, towns, villages, etc

// City labels with dots for low zoom levels.
// The separate attachment keeps the size of the XML down.
#place_label::citydots[type='city'][zoom>=5][zoom<=7][localrank<=3] {
  // explicitly defining all the `ldir` values wer'e going
  // to use shaves a bit off the final project.xml size
  [ldir='N'],[ldir='S'],[ldir='E'],[ldir='W'],
  [ldir='NE'],[ldir='SE'],[ldir='SW'],[ldir='NW'] {
    shield-file: url("img/dot-3.png");
    shield-transform:scale(0.3,0.3);
    shield-unlock-image: true;
    shield-name: @name;
    shield-size: 12;
    shield-face-name: @sans;
    shield-placement: point;
    shield-fill: @text;
    shield-halo-fill: @fill1;
    shield-halo-radius: 2;
    shield-halo-rasterizer: fast;
    shield-margin:30;
    shield-opacity:0.5;
    [zoom=7] { shield-size: 14; }

    [ldir='E'] { shield-text-dx: 5; }
    [ldir='W'] { shield-text-dx: -5; }
    [ldir='N'] { shield-text-dy: -5; }
    [ldir='S'] { shield-text-dy: 5; }
    [ldir='NE'] { shield-text-dx: 4; shield-text-dy: -4; }
    [ldir='SE'] { shield-text-dx: 4; shield-text-dy: 4; }
    [ldir='SW'] { shield-text-dx: -4; shield-text-dy: 4; }
    [ldir='NW'] { shield-text-dx: -4; shield-text-dy: -4; }
  }
}

#place_label[zoom>=8][localrank<=3] {
  text-name: @name;
  text-face-name: @sans;
  text-wrap-width: 80;
  text-wrap-before: true;
  text-fill: @text;
  text-halo-fill: @fill1;
  text-halo-radius: 2;
  text-halo-rasterizer: fast;
  text-size: 10;
  text-line-spacing:-2;
  text-margin:25;

  // Cities
  [type='city'][zoom>=8][zoom<=15] {
  	text-face-name: @sans_md;
    text-size: 15;
    text-line-spacing:-7;

    [zoom>=10] { 
      text-size: 17;
      text-wrap-width: 140;
    }
    [zoom>=12] { 
      text-size: 20;
      text-wrap-width: 180;
    }
    // Hide at largest scales:
    [zoom>=16] { text-name: "''"; }
  }
  
  // Towns
  [type='town'] {
    text-size: 12;    
    text-halo-fill: @fill1;
    text-halo-radius: 1.9;
    [zoom>=12] { text-size: 12; }
    [zoom>=14] { text-size: 16; }
    [zoom>=16] { text-size: 22; }
    // Hide at largest scales:
    [zoom>=18] { text-name: "''"; }
  }
  
  // Villages and suburbs
  [type='village'] {
    text-size: 12;    
    text-halo-fill: @fill1;
    text-halo-radius: 1.9;
    [zoom>=12] { text-size: 10; }
    [zoom>=14] { text-size: 14; }
    [zoom>=16] { text-size: 18; }
  }
  [type='hamlet'],
  [type='suburb'],
  [type='neighbourhood'] {
    text-fill: @fill3;
    text-face-name:	@sans_it;
    text-transform: none;
    text-margin:50;
    text-halo-radius: 2;
    text-character-spacing: 0.5;
    text-size: 12;
    [zoom>=14] { text-size: 14; }
    [zoom>=15] { text-size: 16; text-character-spacing: 1; }
    [zoom>=16] { text-size: 18; text-character-spacing: 2; }
    [zoom>=18] { text-size: 24; text-character-spacing: 3; }
  }
}

// Road labels
#road_label[len>2000][zoom>=12],
#road_label[len>1000][zoom>=15] { 
  text-placement: line;
  text-face-name: @sans;
  text-name: @name;
  text-size: 9;
  text-min-distance: 100;
  text-halo-fill: @land;
  text-halo-radius: 1;
  text-fill: @text;
  [zoom>=17] { text-size: 11;}
}

// Water labels
#marine_label[zoom >= 4][labelrank <= 2] { 
  text-name: @name;
  text-face-name: @sans_bd;
  text-fill: @text;
  text-size: 12;
  text-halo-fill: @water;
  text-halo-radius: 1;
  text-wrap-before: true;
  text-wrap-width: 90;
  [labelrank=1] {
   text-size: 18;
  }
}

#water_label {
  [zoom<=13],
  [zoom>=14][area>500000],
  [zoom>=16][area>10000],
  [zoom>=17] {
    text-name: @name;
    text-face-name: @sans_bd;
    text-fill: @text;
    text-size: 12;
    text-halo-fill: @water;
    text-halo-radius: 1;
    text-wrap-width: 60;
    text-wrap-before: true;
    text-avoid-edges: true;
  }
}

#waterway_label[type='river'][zoom>=13],
#waterway_label[type='canal'][zoom>=14],
#waterway_label[type='stream'][zoom>=15] { 
  text-name: @name;
  text-face-name: @sans_bd;
  text-fill: @text;
  text-min-distance: 60;
  text-size: 10;
  text-halo-fill: @water;
  text-halo-radius: 1;
  text-wrap-before: true;
  text-avoid-edges: true;
  text-placement: line;
}

// Place labels

#poi_label {
  [scalerank<=3][type='Attraction'],
  [scalerank<=2][maki='religious-christian'], [scalerank<=2][maki='religious-jewish'],[scalerank<=2][maki='religious-muslim'], [scalerank<=2][maki='place-of-worship'],

  [scalerank<=2][maki='park'],[scalerank<=2][maki='park2'],
  [scalerank<=2][maki='airport'],[scalerank<=2][maki='airfield'],[scalerank<=2][maki="college"],
  [scalerank<=2][maki='rail'],
  [scalerank<=2][maki='school'],[scalerank<=2][maki='college'],
  [scalerank<=2][maki='hospital'],
  [scalerank<=2][maki='museum'], [scalerank<=2][maki='art-gallery'],
  {
    text-face-name: @sans_bdit;
    text-allow-overlap: false;
    text-name: @name;
    text-size: 14 -  [scalerank];
    text-line-spacing: -2;
    text-min-distance: 50;
    text-wrap-width: 60;
    text-halo-fill: @fill1;
    text-halo-radius: 1;
    text-fill: @text;
  } 
}
