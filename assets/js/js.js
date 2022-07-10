document.addEventListener('DOMContentLoaded', () => {
  const form = document.getElementById('kanjis');
  form.addEventListener('submit', (e) => {
    fetch('/api/process/', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      mode: 'same-origin',
      body: JSON.stringify({ kanjis: form.elements.kanjis.value }),
    })
      .then((resp) => resp.blob()).then((blob) => {
        const file = window.URL.createObjectURL(blob);
        window.location.assign(file);
      }).catch((err) => console.log(err));
    e.preventDefault();
  });
});
