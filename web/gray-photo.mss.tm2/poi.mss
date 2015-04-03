// Mountain peaks //
#poi_label[type='Peak'] {
  marker-width: 5;
  marker-fill: @fill2;
  marker-line-width: 0;  
  text-name: @name;
  text-face-name: @sans_bd;
  text-size: 10;
  text-character-spacing: 0.5;
  text-wrap-width: 55;
  text-fill: @text;
  text-halo-fill: @fill1;
  text-halo-radius: 1.5;
  text-halo-rasterizer: fast;
  text-line-spacing: -4;
  text-dy: 10;
  [zoom>=10] { 
    text-size: 11; 
    text-dy: 11; 
    text-wrap-width: 60;}
  [zoom>=17] { 
    text-size: 12; 
    text-dy: 12; 
    text-wrap-width: 65;}
  [zoom>=19] { 
      text-size: 13; 
      text-dy: 13; 
      text-wrap-width: 70;
  }
}