<template>
  <div class="start_page">
    <input class="search_box" type="text" placeholder="Search" @input="onChangeHandler" v-model="query" @focus="inputSelected = true; onChangeHandler();" @blur="inputSelected = false; onChangeHandler();" autofocus>
    <SuggestionsBox :isSuggestHidden = "isSuggestHidden" :suggestions="suggestions" :query="query"> </SuggestionsBox>
    <h1>{{ msg }}</h1>
    <p>
      For a guide and recipes on how to configure / customize this project,<br>
      check out the
      <a href="https://cli.vuejs.org" target="_blank" rel="noopener">vue-cli documentation</a>.
    </p>
    <h3>Installed CLI Plugins</h3>
    <ul>
      <li><a href="https://github.com/vuejs/vue-cli/tree/dev/packages/%40vue/cli-plugin-babel" target="_blank" rel="noopener">babel</a></li>
      <li><a href="https://github.com/vuejs/vue-cli/tree/dev/packages/%40vue/cli-plugin-eslint" target="_blank" rel="noopener">eslint</a></li>
    </ul>
    <h3>Essential Links</h3>
    <ul>
      <li><a href="https://vuejs.org" target="_blank" rel="noopener">Core Docs</a></li>
      <li><a href="https://forum.vuejs.org" target="_blank" rel="noopener">Forum</a></li>
      <li><a href="https://chat.vuejs.org" target="_blank" rel="noopener">Community Chat</a></li>
      <li><a href="https://twitter.com/vuejs" target="_blank" rel="noopener">Twitter</a></li>
      <li><a href="https://news.vuejs.org" target="_blank" rel="noopener">News</a></li>
    </ul>
    <h3>Ecosystem</h3>
    <ul>
      <li><a href="https://router.vuejs.org" target="_blank" rel="noopener">vue-router</a></li>
      <li><a href="https://vuex.vuejs.org" target="_blank" rel="noopener">vuex</a></li>
      <li><a href="https://github.com/vuejs/vue-devtools#vue-devtools" target="_blank" rel="noopener">vue-devtools</a></li>
      <li><a href="https://vue-loader.vuejs.org" target="_blank" rel="noopener">vue-loader</a></li>
      <li><a href="https://github.com/vuejs/awesome-vue" target="_blank" rel="noopener">awesome-vue</a></li>
    </ul>
  </div>
</template>

<script>
import SuggestionsBox from './SuggestionsBox.vue';

export default {
    name: 'SearchBox',
    props: {
        msg: String
    },
    components: {
        SuggestionsBox
    },
    data() {
        return {
            query: "",
            suggestions: [],
            isSuggestHidden: true,
            inputSelected: false
        };
    },
    methods: {
        onChangeHandler: function() {
            // show the suggestion box, only if the length of the query is > 2
            if (this.query.length >= 2 && this.inputSelected) {
                this.isSuggestHidden = false;
                fetch(`http://localhost:8000/suggest/${this.query}`, {
                    method: 'GET'
                })
                    .then(response => {
                        if (response.ok)
                            response.json()
                                .then(response => this.suggestions = response)
                                .catch(err => console.error(err));
                        else
                            console.error("Suggest served responsed with an unexpected code: " + response.status)
                    });
            } else {
                this.isSuggestHidden = true;
            }
        }
    }
};

</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
h3 {
  margin: 40px 0 0;
}
ul {
  list-style-type: none;
  padding: 0;
}
li {
  display: inline-block;
  margin: 0 10px;
}
a {
  color: #42b983;
}
.start_page {
  margin-top: 50px;
}
.search_box {
  width: 80%;
  max-width: 584px;
  min-width: 380px;
  padding: 12px 44px 12px 24px;
  font-size: 14px;
  line-height: 18px;
  color: var(--foreground-color);
  background-color: var(--background-color);
  background-image: url("~@/assets/search-green.png");
  background-repeat: no-repeat;
  background-size: 18px 18px;
  background-position: 97% center;
  border: 1px solid var(--foreground-color);
  border-radius: 50px;
  position: relative;
  z-index: 3;
}
.search_box::placeholder {
  text-transform: uppercase;
  letter-spacing: 1.3px;
  color: rgba(208, 209, 210, 0.4);
}
.search_box:hover, .search_box:focus {
  outline: none;
  border-color: rgba(208, 209, 210, 0.5);
}
.search_box:hover {
  box-shadow: 0 0 5px 0 rgba(255, 255, 255, 0.1);
}

</style>
