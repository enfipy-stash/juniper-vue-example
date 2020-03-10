<template>
  <div id="app">
    <GameCircle />
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import GameCircle from './components/GameCircle.vue';

@Component({
  components: {
    GameCircle,
  },
})
export default class App extends Vue {}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
  height: 80vh;
}
</style>
