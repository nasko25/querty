<template>
  <div id="app">
    <img alt="Q" id="Q" src="./assets/q.svg">
    <SearchBox msg="Welcome to Your Vue.js App"/>
  </div>
</template>

<script>
import SearchBox from './components/StartPage.vue'

export default {
  name: 'App',
  components: {
    SearchBox
  }
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
  margin-top: 160px;
}
#Q {
  height: 200px;
  width: 200px;
}

.hidden {
    display: none;
}
</style>
