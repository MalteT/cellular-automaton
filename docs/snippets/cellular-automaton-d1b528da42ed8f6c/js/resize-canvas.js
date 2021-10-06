function maximizeCanvas(id) {
  let canvas = document.getElementById(id);
  canvas.width = document.body.clientWidth;
  canvas.height = document.body.clientHeight;
  console.log("resized");
}

export function setResizeHandler(id, callback, timeout) {
  var timer_id = undefined;
  window.addEventListener("resize", function() {
    if(timer_id != undefined) {
      clearTimeout(timer_id);
      timer_id = undefined;
    }
    timer_id = setTimeout(function() {
      timer_id = undefined;
      maximizeCanvas(id);
      callback();
    }, timeout);
  });

  // Do one resize now
  maximizeCanvas(id)
}
