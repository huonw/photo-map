/*global L console */

var SECONDS_PER_DAY = 24 * 3600;
var SECONDS_PER_YEAR = SECONDS_PER_DAY * 365.25;

function sign(x) {
    return x < 0 ? -1 : (x > 0 ? 1 : 0);
}
// allow multiple popups to appear at once.
L.Map = L.Map.extend({
    openPopup: function(popup) {
        // this.closePopup();
        this._popup = popup;
        return this.addLayer(popup).fire('popupopen', {
            popup: this._popup
        });
    }
});

function manage_help() {
    var help = document.getElementById('help-content');
    var help_close = document.getElementById('help-close');
    var help_open = document.getElementById('help-open');

    function set_help(open) {
        window.localStorage.setItem('help-state', open);
        if (open) {
            help.style.display = '';
            help_close.style.display = '';
            help_open.style.display = '';
        } else {
            help.style.display = 'none';
            help_close.style.display = 'none';
            help_open.style.display = 'block';
        }
    }

    help_close.addEventListener('click', function() { set_help(false); });
    help_open.addEventListener('click', function() { set_help(true); });
    set_help(window.localStorage.getItem('help-state') !== 'false');
}

function weighted(x, y, weight) {
    var w =  x + (y - x) * weight;
    return w;
}
function weighted_point(a, b, weight) {
    return L.latLng(weighted(a.lat, b.lat, weight),
                    weighted(a.lng, b.lng, weight));
}

function circle_line_intersection(c, r, p) {
    var cx = c.x, cy = c.y;
    var x0 = p.x - cx, y0 = p.y - cy;
    var x, y;
    if (x0 == 0) {
        x = 0;
        y = sign(y0) * r;
    } else {
        var a = y0/x0;
        x = sign(x0) * r / Math.sqrt(1 + a*a);
        y = a * x;
    }
    return L.point(cx + x, cy + y);
}

function latlngbounds_to_bounds(map, llbounds, zoom) {
    var tl = map.project(llbounds.getNorthWest(), zoom);
    var br = map.project(llbounds.getSouthEast(), zoom);

    return L.bounds(tl, br);
}
function bounds_to_latlngbounds(map, bounds, zoom) {
    var nw = map.unproject(bounds.min, zoom);
    var se = map.unproject(bounds.max, zoom);

    return L.latLngBounds(nw, se);
}
function translate_to_include_bounds(map, lllarge, llsmall, zoom) {
    var large = latlngbounds_to_bounds(map, lllarge, zoom);
    var small = latlngbounds_to_bounds(map, llsmall, zoom);

    var x_translate = 0;
    if (small.min.x < large.min.x) {
        x_translate = small.min.x - large.min.x;
    }
    if (small.max.x > large.max.x) {
        if (x_translate != 0) throw 'small bounds are larger than large bounds';
        x_translate = small.max.x - large.max.x;
    }
    var y_translate = 0;
    if (small.min.y < large.min.y) {
        y_translate = small.min.y - large.min.y;
    }
    if (small.max.y > large.max.y) {
        if (y_translate != 0) throw 'small bounds are larger than large bounds';
        y_translate = small.max.y - large.max.y;
    }

    large.max.x += x_translate;
    large.min.x += x_translate;
    large.max.y += y_translate;
    large.min.y += y_translate;
    return bounds_to_latlngbounds(map, large, zoom);
}

function bounds_at_zoomlevel(bounds, current_zoom, new_zoom) {
    var mean = bounds.max.add(bounds.min).divideBy(2);
    var step = bounds.max.subtract(bounds.min).divideBy(2);

    var multiplier = Math.pow(2, current_zoom - new_zoom);

    return L.bounds(mean.subtract(step.multiplyBy(multiplier)),
                    mean.add(step.multiplyBy(multiplier)));
}
function zoomlevel_for_bounds(map, bounds, current_zoom, llinner) {
    var inner = latlngbounds_to_bounds(map, llinner, current_zoom);

    var inner_corner = inner.max.subtract(inner.min);
    var corner = bounds.max.subtract(bounds.min);

    var x = Math.log(inner_corner.x / corner.x) / Math.log(2);
    var y = Math.log(inner_corner.y / corner.y) / Math.log(2);
    return current_zoom - Math.ceil(Math.max(x, y));
}
function innerpad_bounds(bounds, t, r, b, l) {
    b = b || t;
    l = l || r;

    var dx = bounds.max.x - bounds.min.x;
    var dy = bounds.max.y - bounds.min.y;

    var left = bounds.min.x + Math.min(dx * l[0], l[1]);
    var right = bounds.max.x - Math.min(dx * r[0], r[1]);
    var top = bounds.min.y + Math.min(dy * t[0], t[1]);
    var bottom = bounds.max.y - Math.min(dy * b[0], b[1]);
    return L.bounds([[left, top], [right, bottom]]);
}

function data_loaded(map, clusters, summary_line) {
    var id_to_cluster_info = {};

    var timeline = document.getElementById('timeline');
    var timeline_padding = document.getElementById('timeline-padding');

    var summary = SUMMARY;

    var time_start = summary.times[0][0];
    var time_end = summary.times[summary.times.length - 1][1];
    var time_range = time_end - time_start;

    var detail_lines = L.layerGroup([]);

    var normalise = function(t) {
        return (t - time_start) / time_range;
    };

    var time_to_color = function(time, sat, offset) {
        var normed_time = normalise(time);
        var tick = Math.sin(time / SECONDS_PER_DAY / 3.5 * Math.PI);

        var hue = 330 * normed_time;
        var lightness = offset + 20 * tick;
        //hue = ((time / SECONDS_PER_YEAR) % 1) * 360;
        //lightness = offset;
        return 'hsl(' + hue + ',' + sat + '%,' + lightness + '%)'
    };

    var time_popup = function(bounds, colour, text) {
        var location = L.latLng(bounds.getNorth(), bounds.getCenter().lng);
        var open_popup = function(manual_pan, max_zoom) {
            var pop_opts = {
                autoPan: !manual_pan,
                closeButton: false
            };
            if (manual_pan) {
                var close_to_screen =
                    innerpad_bounds(map.getPixelBounds(), [-1, 0], [-1, 0])
                    .intersects(latlngbounds_to_bounds(map, bounds));

                var view_bounds, zoom;
                if (close_to_screen) {
                    // if the cluster of interest is almost on screen, we pan so
                    // that it just lies inside the screen (with some internal
                    // padding so it's not exactly on the edge) i.e. move as
                    // little as possible, leaving as many existing things on
                    // screen as possible.

                    var current_zoom = map.getZoom();

                    var current_pixel_bounds = map.getPixelBounds();
                    // define the padding
                    current_pixel_bounds = innerpad_bounds(current_pixel_bounds,
                                                           [0.1, 100],
                                                           [0.2, 200]);

                    // compute a zoom level that will ensure that the padded
                    // screen will be able to contain the cluster's points...
                    zoom = zoomlevel_for_bounds(map, current_pixel_bounds, current_zoom,
                                                bounds);
                    // aligning everything correctly requires we use the
                    // final zoom here.
                    zoom = Math.min(zoom, max_zoom);

                    // compute the equivalent pixel size of the screen when we
                    // change zoom, that is, if we zoom in, draw a rectangle
                    // along the screen's borders and then zoom back out, this
                    // variable represents the pixel bounds of that rectangle.
                    var pixel_bounds = bounds_at_zoomlevel(current_pixel_bounds,
                                                           current_zoom, zoom);
                    // Now work out what absolute position the rectangle delineates.
                    var map_bounds = bounds_to_latlngbounds(map, pixel_bounds, current_zoom);
                    // and slide (if necessary) the rectangle so that it
                    // includes `bounds`.
                    view_bounds = translate_to_include_bounds(map, map_bounds,
                                                              bounds, zoom);
                } else {
                    // if the new cluster is too far away we
                    zoom = map.getBoundsZoom(bounds.pad(0.1));
                    view_bounds = bounds;
                }
                // make sure we don't zoom in: the user may've aligned the zoom
                // perfectly.
                var true_zoom = Math.min(zoom, max_zoom);
                map.fitBounds(view_bounds, { maxZoom: true_zoom, animate: true });
            }
            var popup = L.popup(pop_opts)
                        .setLatLng(location)
                        .setContent(text)
                        .openOn(map);
            popup._wrapper.style.background = colour;
            popup._tip.style.background = colour;
        };
        return open_popup;
    };

    var render_cluster = function(cluster, times) {
        var colour = time_to_color(cluster.mean_time, 100, 60);
        var start = times[0];
        var end = times[times.length - 1];

        // draw the line connecting the photos
        var lineopts = {
            color: colour,
            opacity: 1.0,
            weight: 2,
            clickable: true
        };
        var line = L.polyline(cluster.coords, lineopts);
        detail_lines.addLayer(line);

        // draw the circle around that line
        var line_bounds = line.getBounds();
        var radius = line_bounds.getNorthEast().distanceTo(line_bounds.getSouthWest()) / 2 * 1.05;
        var coords = line_bounds.getCenter();

        var circle_opts = {
            color: colour,
            opacity: 1.0,
            weight: 5,
            fill: false
        };
        var circle = L.circle(coords, radius, circle_opts).addTo(map);

        // make the small rectangle on the timeline
        var clicker = document.createElement('div');
        clicker.id = 'timeline-clicker-' + cluster.id;
        clicker.classList.add('timeline-clicker');
        var start_pct = normalise(start) * 100;
        var end_pct = normalise(end) * 100;
        clicker.style.width = (end_pct - start_pct) + '%';
        clicker.style.marginLeft = start_pct + '%';
        timeline.appendChild(clicker);

        // make the marker that labels the circle
        var start_date = new Date(start * 1000).toDateString();
        var end_date = new Date(end * 1000).toDateString();
        var marker_text = start_date == end_date ? start_date : start_date + ' - ' + end_date;
        var bounds = circle.getBounds();
        var make_popup = time_popup(bounds, colour, marker_text);

        // set the click handlers
        var f = function(max_zoom) {
            return function() {
                make_popup(true, Math.max(max_zoom, map.getZoom()));
                line._container.parentNode.classList.add('fade-all');
                circle._path.classList.add('dont-fade');
                line._path.classList.add('dont-fade');
            };
        };
        clicker.addEventListener('click', f(15));
        line.addEventListener('click', f(null));
        circle.addEventListener('click', f(null));

        return { coords: coords, radius: radius };
    }

    var width = document.body.clientWidth;
    var height = document.body.clientHeight;
    //var timeline_width = Math.max(width, 2 * time_range / SECONDS_PER_DAY);
    //timeline_padding.style.width = timeline_width + 'px';

    for (var year_start = Math.ceil(time_start/SECONDS_PER_YEAR)*SECONDS_PER_YEAR;
         year_start < time_end;
         year_start += SECONDS_PER_YEAR) {
        var which_year = 1970 + Math.floor(year_start / SECONDS_PER_YEAR);
        var year = document.createElement('div');
        year.classList.add('timeline-year');
            year.textContent = which_year;
            year.style.marginLeft = (normalise(year_start) * 100) + '%';
        timeline.appendChild(year);
    }

    for (var i = 0; i < clusters.length; i++) {
        var cluster = clusters[i];
        id_to_cluster_info[cluster.id] = cluster;
    }

    var prev = null;

    for (var i = 0; i < summary.coords.length; i++) {
        var id = summary.ids[i];
        var cluster = id_to_cluster_info[id];

        var current = {
            id: id,
            coords: L.latLng(summary.coords[i]),
            times: summary.times[i],
            radius: 0
        };

        if (cluster && cluster.times.length > 1) {
            var ret = render_cluster(cluster, current.times);
            current.coords = ret.coords;
            current.radius = ret.radius;
        }

        if (prev !== null) {
            var seg_start = prev.times[1];
            var seg_end = current.times[0];
            var dt = (seg_end - seg_start) / SECONDS_PER_DAY;
            var dist = current.coords.distanceTo(prev.coords) / 1000;
            var speed = dist / (dt * 24);

            var options = {
                weight: 3,
                //dashArray: [1, 5],
                opacity: 1.0,
                clickable: false
            };
            var sat = 100;
            if (dt > 30 || dist > 500 || speed > 150) {
                options.dashArray = [1, 10];
                //options.weight /= 1.5
            }

            var ZOOM = 18;
            var prev_tweak = L.latLng(prev.coords.lat + 1, prev.coords.lng);
            var cur_tweak = L.latLng(current.coords.lat + 1, current.coords.lng);

            var prev_center = map.project(prev.coords, ZOOM);
            var cur_center = map.project(current.coords, ZOOM);
            var prev_scale = prev_center.distanceTo(map.project(prev_tweak, ZOOM)) /
                prev.coords.distanceTo(prev_tweak);
            var cur_scale = cur_center.distanceTo(map.project(cur_tweak, ZOOM)) /
                current.coords.distanceTo(cur_tweak);

            var start_point = circle_line_intersection(prev_center, prev.radius * prev_scale,
                                                       cur_center);
            var end_point = circle_line_intersection(cur_center, current.radius * cur_scale,
                                                     prev_center);

            var start = map.unproject(start_point, ZOOM);
            var end = map.unproject(end_point, ZOOM);

            var prev_ = start;

            var weight = Math.ceil(7/2 * dt);
            weight = 1;
            for (var j = 1; j <= weight; j++) {
                var current_ = weighted_point(start, end, j / weight);
                var time = weighted(seg_start, seg_end, (j - 0.5) / weight);
                var normed_time = normalise(time);

                options.color = time_to_color(time, sat, 60);
                L.polyline([prev_, current_], options).addTo(map);
                prev_ = current_;
            }
        }
        prev = current;
    }

    var detail_visible = false;
    var zoom_show_detail = function() {
        if (map.getZoom() >= 6) {
            if (!detail_visible) {
                map.addLayer(detail_lines);
            }
            detail_visible = true;
        } else {
            if (detail_visible) {
                map.removeLayer(detail_lines);
            }
            detail_visible = false;
        }
    };

    zoom_show_detail();
    map.addEventListener('zoomend', zoom_show_detail);

    if (!!summary_line) {
        map.removeLayer(summary_line);
    }
};



window.addEventListener('load', function() {
    manage_help();

    var map = L.map('map');

    var tiles ='https://{s}.tiles.mapbox.com/v4/huon.ljnfkno2/{z}/{x}/{y}.png?access_token=pk.eyJ1IjoiaHVvbiIsImEiOiI2SjBWVnFnIn0.Z77zTYr8S455QVmC8ROBog';
    var attrib = "<a href='https://www.mapbox.com/about/maps/' target='_blank'>&copy; Mapbox &copy; OpenStreetMap</a> <a class='mapbox-improve-map' href='https://www.mapbox.com/map-feedback/' target='_blank'>Improve this map</a>";

    L.tileLayer(tiles, {
        attribution: attrib,
        maxZoom: 20
    }).addTo(map);

    var summary_line = L.polyline(SUMMARY.coords, { color: 'white', opacity: 1.0, weight: 1 })
                       .addTo(map);

    var parsed_hash = L.Hash.prototype.parseHash(window.location.hash);
    var set_bounds = parsed_hash ?
        function() { map.setView(parsed_hash.center, parsed_hash.zoom); } :
    function() { map.fitBounds(summary_line.getBounds()); };
    set_bounds();
    var _hash = L.hash(map);

    L.control.scale({ imperial: false }).addTo(map);

    map.addEventListener('popupclose', function() {
        ['fade-all', 'dont-fade'].forEach(function(klass) {
            var elements = document.getElementsByClassName(klass);
            elements = Array.prototype.slice.call(elements);
            Array.prototype.forEach.call(elements, function(e) {
                e.classList.remove(klass);
            });
        });
    });

    var mini_tiles = L.tileLayer(tiles, {
        attribution: attrib,
        maxZoom: 12,
        minZoom: 2
    });
    var minimap = L.control.minimap(mini_tiles, {
        autoToggleDisplay: true,
        position: 'topright',
        aimingRectOptions: { color: 'white', weight: 2, opacity: 1.0 },
        zoomLevelOffset: -8
    }).addTo(map);
    var mini_summary = L.polyline(SUMMARY.coords, { color: 'white', opacity: 0.9, weight: 1 })
                       .addTo(minimap._miniMap);
    set_bounds();

    var clustersReq = new XMLHttpRequest();

    clustersReq.onload = function() {
        var clusters = JSON.parse(this.responseText);
        data_loaded(map, clusters, summary_line);
    }

    clustersReq.open('get', 'data/clusters.json', true);
    clustersReq.send();
});
