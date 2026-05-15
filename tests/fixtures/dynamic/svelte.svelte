<script>
  let isActive = true;
  let size = 'large';
  let isError = false;
</script>

<div class="static">
  <!-- Svelte class directives -->
  <div class:active={isActive} class:large={size === 'large'}>
    Class directive
  </div>

  <!-- Svelte shorthand class directive -->
  <div class:isError>
    Shorthand
  </div>

  <!-- Template expressions in class attribute -->
  <div class="base {isActive ? 'active' : ''} {isError ? 'error' : ''}">
    Concatenation
  </div>

  <!-- Dynamic class with curly braces -->
  <div class={isActive ? 'bg-green' : 'bg-red'}>
    Ternary in curly
  </div>
</div>

<style>
  .active { color: green; }
  .large { font-size: 20px; }
  .error { color: red; }
</style>
