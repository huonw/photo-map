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

window.addEventListener('load', function() {
    var utc_offset = new Date().getTimezoneOffset() * 60 * 1000;
    var map = L.map('map');

    var since = 0;
    var points = [];
    var layers = L.layerGroup();
    var following = window.localStorage.getItem('following') !== 'false';
    var location_marker = L.marker([0, 0]).addTo(map);
    var set_following = function(follow) {
        following = follow;
        window.localStorage.setItem('following', follow);
    };
    location_marker.addEventListener('click', function() {
        map.panTo(location_marker.getLatLng());
        set_following(true);
    });
    var cancel_following = function() { set_following(false) };

    var set_cancellation = function(on) {
        var func = on ? 'addEventListener' : 'removeEventListener';
        map[func]('dragstart', cancel_following);
        map[func]('zoomstart', cancel_following);
    }
    set_cancellation(true);

    function update_points(new_points, pan) {
        map.removeLayer(layers);
        layers.clearLayers();

        points = points.concat(new_points);
        var most_recent = points[points.length - 1];
        var mr_loc = L.latLng(most_recent.lat,most_recent.lon);
        location_marker.setLatLng(mr_loc);
        if (pan) {
            set_cancellation(false);
            map.panTo(mr_loc);
            set_cancellation(true);
        }
        since = most_recent.timestamp;

        var now = Date.now() + utc_offset;

        var LARGEST = 10;
        var HALF_LIFE = 1000 * 30;
        var MIN = 1;
        var prev = points[0];
        for (var i = 1; i < points.length; i++) {
            var cur = points[i];
            var weight = MIN + (LARGEST - MIN) * Math.pow(0.5,
                                                          Math.max(0, now - cur.timestamp)
                                                              / HALF_LIFE);
            var options = { weight: weight, opacity: 1.0, clickable: false };
            layers.addLayer(L.polyline([[prev.lat, prev.lon],
                                        [cur.lat, cur.lon]],
                                       options));
            prev = cur;
        }
        map.addLayer(layers);

    }
    update_points(POINTS, false);

    var tiles ='https://{s}.tiles.mapbox.com/v4/huon.ljnfkno2/{z}/{x}/{y}.png?access_token=pk.eyJ1IjoiaHVvbiIsImEiOiI2SjBWVnFnIn0.Z77zTYr8S455QVmC8ROBog';
    var attrib = "<a href='https://www.mapbox.com/about/maps/' target='_blank'>&copy; Mapbox &copy; OpenStreetMap</a> <a class='mapbox-improve-map' href='https://www.mapbox.com/map-feedback/' target='_blank'>Improve this map</a>";

    L.tileLayer(tiles, {
        attribution: attrib,
        maxZoom: 20
    }).addTo(map);

    var parsed_hash = L.Hash.prototype.parseHash(window.location.hash);
    var _hash = L.hash(map);
    if (following || !parsed_hash) {
        var zoom = parsed_hash ? parsed_hash.zoom : 15;
        map.setView(location_marker.getLatLng(), zoom)
    } else {
        map.setView(parsed_hash.center, parsed_hash.zoom);
    }

    L.control.scale({ imperial: false }).addTo(map);

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

    function run_request() {
        var pointsReq = new XMLHttpRequest();

        pointsReq.onload = function() {
            update_points(JSON.parse(this.responseText), following);
            window.setTimeout(run_request, 2000);
        }

        pointsReq.open('get', '/live/points.json?' + since, true);
        pointsReq.send();
    }

    run_request();
});
