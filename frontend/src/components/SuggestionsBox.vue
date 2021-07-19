<template>
    <div id = "suggestions" :class = "{hidden: isSuggestHidden}">
        <ul id = "suggestions-list">
            <li class = "suggestion" v-for = "(suggestion, index) in suggestions.slice(0, MAX_SUGGESTION_COUNT)" :key = "index" :class="{ 'onhover' : (index === focusSuggestion), 'onhover_last_child' : ((index === focusSuggestion) && (index === (suggestions.length - 1))) }" @mouseover="mouseOverSuggestion(index)">
                {{ getNonBoldPartOfQuery(suggestion, query) }}<b>{{ getBoldPartOfQuery(suggestion, query) }}</b>
            </li>
        </ul>
    </div>
</template>

<script>

export default {
    name: 'SuggestionsBox',
    props: {
        suggestions: Array,
        isSuggestHidden: Boolean,
        query: String,
        focusSuggestion: Number,
        MAX_SUGGESTION_COUNT: Number
    },
    //data() {
    //    return {
    //        focusSuggestionNumber: this.focusSuggestion
    //    }
    //},
    //watch: {
    //    'focusSuggestion': function(newFocusSuggestion) {
    //        console.log("focus suggestion changed; new value: " + newFocusSuggestion);
    //        this.focusSuggestionNumber = newFocusSuggestion;
    //    }
    //},
    methods: {
        mouseOverSuggestion(index) {
            // notify the parent component that the focusSuggestionNumber variable should be changed
            this.$emit('focusSuggestionChange', index);
        },
        getNonBoldPartOfQuery(suggestion, query) {
            const split_query = query.split(/[^A-Za-z\d]/).filter(element => element !== "");
            return suggestion.startsWith(split_query[split_query.length - 1]) ? query : "";
        },
        getBoldPartOfQuery(suggestion, query) {
            const split_query = query.split(/[^A-Za-z\d]/);
            return suggestion.startsWith(split_query[split_query.length - 1]) ? suggestion.replace(split_query[split_query.length - 1], "") : suggestion;
        }
    }
};

</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

#suggestions {
    height: auto;
    width: 80%;
    max-width: 654px;
    min-width: 380px;

 --suggestions-background: #124;

    background-color: var(--suggestions-background);
    position: absolute;
    margin-left: auto;
    margin-right: auto;
    left: 0;
    right: 0;
    top: 435px;
    z-index: 0;
    border-radius: 0 0 20px 20px;
}

#suggestions-list {
    background-color: var(--suggestions-background);
    margin-top: 1.5em;
    margin-bottom: 0.4em;
    padding-left: 0;
    border-radius: 0 0 20px 20px;
}

.suggestion {
    background-color: var(--suggestions-background);
    list-style: none;
    text-align: left;
    padding-top: 0.3em;
    padding-bottom: 0.3em;
    padding-left: 20px;
    border-radius: 0 0 20px 20px;
}

.suggestion b {
    background: transparent;
}

.onhover {
    background-color: #123;
    background-color: #000;
    border-radius: 0;
    cursor: pointer;
}

.onhover_last_child {
    border-radius: 0 0 11px 11px;
}

.suggestion::before {
    content: " ";
    display: inline-block;
    height: 13px;
    width: 13px;
    background-image: url("~@/assets/search-green.png");
    background-size:contain;
    background-repeat:no-repeat;
    position: relative;
    top: 1.5px;
    padding-right: 10px;
}
</style>
