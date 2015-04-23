// allow multiple popups to appear at once.
function sign(x) {
    return x < 0 ? -1 : (x > 0 ? 1 : 0);
}
L.Map = L.Map.extend({
    openPopup: function(popup) {
        // this.closePopup();
        this._popup = popup;
        return this.addLayer(popup).fire('popupopen', {
            popup: this._popup
        });
    }
});

window.addEventListener('load', function() {
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
    var SECONDS_PER_DAY = 24 * 3600;
    var SECONDS_PER_YEAR = SECONDS_PER_DAY * 365.25;
    var WIDTH_STEPS = [60, 30, 14, 7, 3.5, 0];

    var timeline = document.getElementById('timeline');
    var timeline_padding = document.getElementById('timeline-padding');

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

    var id_to_cluster_line = {};
    var id_to_cluster_marker = {};
    var id_to_cluster_info = {};


    var clustersReq = new XMLHttpRequest();
    var clusters = null;

    var data_loaded = function() {
        if (!clusters) { throw 'data not yet loaded' };
        var summary = SUMMARY;

        var time_start = summary.times[0][0];
        var time_end = summary.times[summary.times.length - 1][1];
        var time_range = time_end - time_start;
        var normalise = function(t) {
            return (t - time_start) / time_range;
        };

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


        var time_to_color = function(time, sat, offset) {
            var normed_time = normalise(time);
            var tick = Math.sin(time / SECONDS_PER_DAY / 3.5 * Math.PI);

            var hue = 330 * normed_time;
            var lightness = offset + 20 * tick;
            //hue = ((time / SECONDS_PER_YEAR) % 1) * 360;
            //lightness = offset;
            return 'hsl(' + hue + ',' + sat + '%,' + lightness + '%)'
        }

        var time_marker = function(bounds, time, text, options) {
            var color = time_to_color(time, 50, 60);
            options.color = color;
            var location = L.latLng(bounds.getNorth(), bounds.getCenter().lng);
            var marker = L.circleMarker(location, options).addTo(map);
            var open_popup = function(manual_pan) {
                var pop_opts = {
                    autoPan: true,
                    closeButton: false
                };
                if (manual_pan) {
                    var zoom = Math.min(map.getBoundsZoom(bounds), map.getZoom());
                    map.fitBounds(bounds, { maxZoom: zoom, animate: true });
                    pop_opts.autoPan = false;
                }
                marker.bindPopup(text, pop_opts).openPopup();
                var popup = marker._popup;
                popup._wrapper.style.background = color;
                popup._tip.style.background = color;
            };
            marker.addEventListener('click', function() { open_popup(true) });
            marker.make_popup = open_popup;
            return marker;
        };

        var cluster_indicator = function(id, line, circle, times, options) {
            var start = times[0];
            var end = times[times.length - 1];
            var start_date = new Date(start * 1000).toDateString();
            var end_date = new Date(end * 1000).toDateString();
            var text = start_date == end_date ? start_date : start_date + ' - ' + end_date;

            options.radius = 0;
            var bounds = circle.getBounds();
            var marker = time_marker(bounds, (start + end) / 2, text, options);
            id_to_cluster_marker[id] = marker;

            var clicker = document.createElement('div');
            clicker.id = 'timeline-clicker-' + id;
            clicker.classList.add('timeline-clicker');
            clicker.style.width = (normalise(end) - normalise(start)) * 100 + '%';
            clicker.style.marginLeft = normalise(start) * 100 + '%';
            timeline.appendChild(clicker);

            var f = function() {
                marker.make_popup(true)
                line._container.parentNode.classList.add('fade-all');
                circle._path.classList.add('dont-fade');
                line._path.classList.add('dont-fade');
            };
            clicker.addEventListener('click', f);
            line.addEventListener('click', f);
            circle.addEventListener('click', f);
        }

        var detail_lines = L.layerGroup([]);

        for (var i = 0; i < clusters.length; i++) {
            var cluster = clusters[i];

            var lineopts = {
                color: time_to_color(cluster.mean_time, 50, 60),
                opacity: 1.0,
                weight: 2,
                clickable: true
            };
            var line = L.polyline(cluster.coords, lineopts);
            detail_lines.addLayer(line);
            id_to_cluster_line[cluster.id] = line;
            id_to_cluster_info[cluster.id] = cluster;

            var start_pos = normalise(cluster.times[0]) * 100;
            var end_pos = normalise(cluster.times[cluster.times.length - 1]) * 100;
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

        var prev = null;
        for (var i = 0; i < summary.coords.length; i++) {
            var id = summary.ids[i];
            var line = id_to_cluster_line[id];
            var line_bounds = line.getBounds();
            var radius = line_bounds.getNorthEast().distanceTo(line_bounds.getSouthWest()) / 2 * 1.05;

            var current = {
                id: id,
                coords: line_bounds.getCenter(),//L.latLng(summary.coords[i]),
                times: summary.times[i],
                radius: radius
            };

            var circle_opts = {
                color: time_to_color(id_to_cluster_info[current.id].mean_time, 100, 60),
                opacity: 1.0,
                weight: 5,
                fill: false,
            };
            if (id_to_cluster_info[id].times.length > 1) {
                var circle = L.circle(current.coords, radius, circle_opts).addTo(map);
                cluster_indicator(id, line, circle, current.times, {});
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
                for (var j = 0; j < WIDTH_STEPS.length; j++) {
                    if (dt > WIDTH_STEPS[j]) {
                        //options.weight =  2 * Math.sqrt(j + 1);
                        break;
                    }
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

        if (!!summary_line) {
            map.removeLayer(summary_line);
        }
    };

    clustersReq.onload = function() {
        clusters = JSON.parse(this.responseText);
        data_loaded();
    }

    clustersReq.open('get', 'data/clusters.json', true);
    clustersReq.send();
});
