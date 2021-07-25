<template>
  <div id="app">
    <router-view></router-view>
  </div>
</template>

<script>

export default {
  name: 'App',
}
</script>

<style>
:root {
    --background-color: #123;
    --foreground-color: #d0d1d2;
}

* {
  background-color: var(--background-color);
}
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: var(--foreground-color);
}
#Q {
  height: 200px;
  width: 200px;
}

.hidden {
    display: none;
}
</style>
