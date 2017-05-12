

function init(state, el) {
  console.log("[initializePanZoom]", el);
  window.requestAnimationFrame(function(_) {
    console.log("[initializePanZoom:animation]");
    state[el] = svgPanZoom(el, {dblClickZoomEnabled: false});
    state[el].resize();
    state[el].center();
    state[el].fit();
});
}

function update(state, el) {
  console.log("[updateBoundingBox]", el);
  window.requestAnimationFrame(function(_) {
    console.log("[updateBoundingBax:animate]", el);
    state[el].updateBBox();
    state[el].resize();
    state[el].center();
    state[el].fit();
  });
  
}

function PanZoom_initializePorts(app) {
  var state = {};
  app.ports.initializePanZoom.subscribe(function(s) {init(state, s)});
  app.ports.updateBoundingBox.subscribe(function(s) {update(state, s)});
}
