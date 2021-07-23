<template>
  <div>
    <input class="search_box" type="text" placeholder="Search" @input="onChangeHandler" v-model="query" @focus="inputSelected = true; onChangeHandler();" @blur="inputSelected = false; onChangeHandler();"  v-on:keydown.up="arrowUpHandler" v-on:keydown.down="arrowDownHandler" v-on:keydown.right="arrowRightHandler" v-on:keydown.enter="search"/>
    <SuggestionsBox :isSuggestHidden = "isSuggestHidden" :suggestions="suggestions" :query="query" :focusSuggestion="focusSuggestion" :MAX_SUGGESTION_COUNT="MAX_SUGGESTION_COUNT" @focusSuggestionChange="onFocusSuggestionChange" @searchSuggestion="onSearchSuggestion" ref="suggestionsBoxRef"> </SuggestionsBox>
  </div>
</template>

<script>
import SuggestionsBox from './SuggestionsBox.vue';

export default {
    name: 'SearchBar',
    components: {
        SuggestionsBox
    },
    data() {
        return {
            query: "",
            suggestions: [],
            isSuggestHidden: true,
            inputSelected: false,
            focusSuggestion: 0,         // which suggestion is focused (1-7 for the suggestions, 0 for none; it should loop around!)
            MAX_SUGGESTION_COUNT: 7,
            SEARCH_SUGGESTION: false    // flag that controls whether pressing <enter> will search for the focused suggestion or the input query
                                        //  by default pressing <enter> will search for the query, but if a user focuses a different selection, <enter> will search for that selection
        };
    },
    methods: {
        onChangeHandler: function() {
            // whenever a new letter is typed, <enter> will search for the new query, instead for the selected suggestion
            this.SEARCH_SUGGESTION = false;

            // show the suggestion box, only if the length of the query is > 2
            if (this.query.length >= 2 && this.inputSelected) {
                this.isSuggestHidden = false;
                fetch(`http://${window.location.hostname}:8000/suggest/${encodeURIComponent(this.query)}`, {
                    method: 'GET'
                })
                    .then(response => {
                        if (response.ok)
                            response.json()
                                .then(response => {
                                        this.suggestions = response;
                                        // add the query to the beginning of the suggestions list, so that it would appear in the
                                        //  suggestions box
                                        const split_query = this.query.split(/[^A-Za-z\d]/);
                                        if (!this.suggestions.slice(0, this.MAX_SUGGESTION_COUNT).includes(split_query[split_query.length - 1]))
                                            this.suggestions.unshift(this.query);
                                    })
                                .catch(err => console.error(err));
                        else
                            console.error("Suggest served responsed with an unexpected code: " + response.status)
                    })
                    .catch(err => {
                            console.error(err);
                    });
            } else {
                this.isSuggestHidden = true;
            }
        },
        arrowUpHandler: function(event) {
            console.log("key")
            // prevent up arrow to move the cursor
            event.preventDefault();

            if (!this.isSuggestHidden) {
                this.SEARCH_SUGGESTION = true;
                this.focusSuggestion = (this.focusSuggestion - 1) % Math.min(this.MAX_SUGGESTION_COUNT, this.suggestions.length);  // because there are maximum of 7 suggestions, and the variable should loop around
                if (this.focusSuggestion === -1) this.focusSuggestion = Math.min(this.MAX_SUGGESTION_COUNT, this.suggestions.length) - 1;
                else if (this.focusSuggestion < 0) this.focusSuggestion = this.focusSuggestion * (-1);
                console.log("focus " + this.focusSuggestion + " suggestions size (%) " + Math.min(this.MAX_SUGGESTION_COUNT, this.suggestions.length));
            }
        },
        arrowDownHandler: function(event) {
            console.log("key down");
            // prevent down arrow to move the cursor
            event.preventDefault();

            if (!this.isSuggestHidden) {
                this.SEARCH_SUGGESTION = true;
                this.focusSuggestion = (this.focusSuggestion + 1) % Math.min(this.MAX_SUGGESTION_COUNT, this.suggestions.length);
                console.log("focus " + this.focusSuggestion);
            }
        },
        arrowRightHandler: function(event) {
            // get the cursor position
            const selectionStart = event.srcElement.selectionStart;
            const selectionEnd = event.srcElement.selectionEnd;

            // prevent the right arrow to move the cursor
            //  but only if the cursor is on the last character of the query
            //      check if the start and end of the selection are equal and if the cursor is at the end of the query
            if (selectionStart === selectionEnd && selectionEnd === this.query.length) {
                console.log("cursor at the end of the query");
                event.preventDefault();

                // auto complete the query with the focused suggestion
                console.log("key right");
                if (!this.isSuggestHidden) {
                    const suggestionsBoxRef = this.$refs.suggestionsBoxRef;
                    // when clicking on the right arrow, replace the query with the suggestion
                    //  use the methods from the SuggestionsBox component to construct the new query
                    //  (the same way it is done in the SuggestionsBox component)
                    this.query = suggestionsBoxRef.getNonBoldPartOfQuery(this.suggestions[this.focusSuggestion], this.query) + suggestionsBoxRef.getBoldPartOfQuery(this.suggestions[this.focusSuggestion], this.query);
                    this.onChangeHandler();
                }
            }
        },
        onFocusSuggestionChange: function(newFocusSuggestion) {
            this.SEARCH_SUGGESTION = true;
            this.focusSuggestion = newFocusSuggestion;
        },
        redirectToResults: function(query) {
            // this only matters if you search from the results page
            // this.isSuggestHidden = true;
            this.query = query;
            // unfocus the input field and set its value to the query
            document.getElementsByClassName("search_box")[0].blur();

            // redirect to /results
            this.$router.push({ path: '/results', query: { q: query } });
        },
        onSearchSuggestion: function(suggestionToSearch) {
            const query = this.suggestions[suggestionToSearch];
            //this.$router.push({ path: '/results', query: { q: query } })
            this.redirectToResults(query);
        },
        search: function(event) {
            event.preventDefault();
            console.log("enter");
            const query = this.SEARCH_SUGGESTION ? this.suggestions[this.focusSuggestion] : this.query;
            this.redirectToResults(query);
        }
    },
    mounted: function() {
        if (this.$route.path === "/results") {
            // if the search bar is mounted on the results page, set the query to the "q" url parameter
            this.query = this.$route.query.q;
        } else {
            // otherwise focus the search input field, because the search bar was mounted on the start page
            document.getElementsByClassName("search_box")[0].focus();
        }
    }
};
</script>

<style scoped>
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
