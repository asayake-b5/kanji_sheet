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
      .then((resp) => resp.text()).then((resp) => {
        console.log(resp);
      }).catch((err) => console.log(err));
    e.preventDefault();
  });
});
