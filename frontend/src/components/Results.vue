<template>
<div>
    <a href="/"> <img alt="Q" id="Q" src="../assets/q.svg"/> </a>
    <SearchBar/>
    <ul id = "results-list">
        <li class = "result" v-for="(result, index) in results" :key = "index">
            <!-- &hellip; is only added if the text was truncated -->
            <a class = "result-title" :href="result.url" @mouseover="underline" @mouseleave="deunderline"> {{ result.title === undefined || result.title === "" ? truncate(result.text, 25) + (result.text.length > 25 ? "&hellip;" : "") : result.title }} </a>
            <br/>
            <a class = "result-url" :href="result.url" @mouseover="underlineTitle" @mouseleave="removeUnderline"> {{ result.url }} </a>
            <br/>
            <div class = "result-text"> <p v-html="getText(result)"></p> </div>
            <br/> <br/>
        </li>
    </ul>
    <!-- If there is an error, it will be displayed in this div -->
    <div id = "error" :class = "{ hidden : err === null }"> <p> {{ err }} </p> </div>
</div>
</template>

<script>
import SearchBar from './SearchBar.vue';

// helper function to get the length of the last word in the result's text field, so that the last word would not be cut (unless it is too long)
function getEndOfText(initial_index, text) {
    // the initial index is 200, so the last word ends after at least 200 characters
    let index = initial_index + 200;
    let char_at_index = text.charCodeAt(index);
    while ( (
                (char_at_index > 47 && char_at_index < 58) ||   // 0-9
                (char_at_index > 64 && char_at_index < 91) ||   // A-Z
                (char_at_index > 96 && char_at_index < 123)     // a-z
            ) &&
            // only cut last word if it makes the whole text > (200 + query.length + 490)
            (index < initial_index + 490)
    ) {
        index++;
        char_at_index = text.charCodeAt(index);
    }
    return index;
}

export default  {
    name: 'Results',
    components: {
        SearchBar
    },
    data() {
        return {
            results: [],
            err: null
        }
    },
    beforeMount() {
        this.fetchResults();
    },
    methods: {
        fetchResults: function() {
            const query = this.$route.query.q;
            if (query === undefined || query === "") {
                console.error("Empty query.")
                // indicate error
                this.err = "Empty query";
                this.results = [];
            }
            else {
                this.err = null;
            }
            console.log("query:", query);

            // TODO also sort by rank
            fetch(`http://${window.location.hostname}:8000/query/${encodeURIComponent(query)}`, {
                    method: 'GET'
                })
                    .then(response => {
                        if (response.ok)
                            response.json()
                                .then(response => this.results = response)
                                .catch(err => console.error(err));
                        else
                            console.error("Suggest served responsed with an unexpected code: " + response.status)
                    }).catch(err => {
                        this.err = "Failed to fetch the query. Try again later.";
                        console.error(err);
            });
        },
        truncate(str, max_len) {
            return (str.length > max_len) ? str.substr(0, max_len - 1) : str;
        },
        underline(event) {
            event.target.style.textDecoration = "underline";
        },
        deunderline(event) {
            event.target.style.textDecoration = "none";
        },
        underlineTitle(event) {
            event.target.previousSibling.previousSibling.style.textDecoration = "underline";
        },
        removeUnderline(event) {
            event.target.previousSibling.previousSibling.style.textDecoration = "none";
        },
        getText(result) {
            /* TODO do that in the backend */
            const split_query = this.$route.query.q.split(/[^A-Za-z\d]/).filter(entry => entry.trim() != '');
            // initialize a variable that is assigned inside the if condition and keeps track whether result.metadata includes "description"
            let RES_INCL_DESC;
            if (result.metadata && ((RES_INCL_DESC = result.metadata.includes("description")) === true || result.metadata.includes("og:description"))) {
                // get the value of "description" or "og:description" (depending on which is present on the website)
                //  if both are present, use "description"
                let description = result.metadata[RES_INCL_DESC ? result.metadata.indexOf("description") + 1 : result.metadata.indexOf("og:description") + 1];
                split_query.forEach(query => {
                    const query_regex = new RegExp(query, "ig"); // ignore case
                    description = description.replaceAll(query_regex, match => `<b>${match}</b>`);
                });
                return description;
            }
            else {
                let result_text = result.text;
                // variables that keep track if the beginning and end of the result text are cut (in order to add "..." if the result text was cut)
                let result_beginning_cut = false;
                let result_end_cut = false;
                split_query.forEach(query => {
                    //const query_regex = new RegExp(query, "ig"); // ignore case
                    //if (result_text.search(query_regex) === -1)
                    //    return;
                    // split by whitespace characters: .split(/\s+/)

                    // TODO maybe don't subtract 200 from the index of query in the result_text if the query is in the beginning of the sentence
                    //  this 200 characters limit can be if the sentence is too long and there are more than 200 characters before the index of the search query
                    //  BUT how can you reliably determine the beginning of a sentence?
                    let result_start = result_text.search(new RegExp(query, "ig")) - 50;
                    // if there are less than 200 characters before the query, then the result text is not cut
                    if (result_start <= 0) {
                        result_start = 0;
                    }
                    // otherwise the beginning of the result text was cut
                    else {
                        result_beginning_cut = true;
                    }

                    // if the calculated end of text is longer than the actual length of result_text, then the text should be fully
                    //  displayed and cannot be cut
                    let result_end = getEndOfText(50 + result_start + Math.max(query.length, 100), result_text);
                    if (result_text.length <= result_end) {
                        result_end = result_text.length;
                    }
                    // otherwise the end of the result text was cut
                    else {
                        result_end_cut = true;
                    }

                    result_text = result_text.substring(result_start, result_end);
                    //console.log(result_text.search(query_regex), query, result_text, result_text.substr(result_text.search(query_regex), query.length))
                });

                const query_regex = new RegExp(split_query.join("|"), "ig");

                // remove leading and trailing spaces
                // result_text = result_text.trim();

                // add "..." to the beginning and/or end of result_text if the text was cut
                if (result_beginning_cut)
                    result_text = "&hellip; " + result_text;
                if (result_end_cut)
                    result_text += " &hellip;";

                result_text = result_text.replaceAll(query_regex, match => `<b>${match}</b>`);
                return result_text;
            }
        }
    },
    watch:{
        $route (){
            // whenever the url changes (meaning the query is different) fetch the new results from the server
            this.fetchResults();
        }
    }
    //beforeMount() {
    //    this.fetchResults();
    //}
}
</script>

<style scoped>
#error {
    /* color: #e4b9c3; */
    color: #a3c9ef;
    font-size: 2em;
    font-family: sans-serif, "Trebuchet MS";
    margin-top: 3em;
    text-align: center;
}

#results-list {
    margin-top: 60px;
    display: inline-block;
    float: left;
    clear: both;
}

.result {
    list-style: none;
}

.result-title {
    float: left;
    text-decoration: none;
    /* color: #5a55aa; */
    color: #5b5fe2;
}

.result-title:visited {
    color: #aeb1f1;
}

.result-url {
    float: left;
    padding-top: 0.1em;
}

.result-url, .result-url:hover, .result-url:visited, .result-url:active {
    color: #cce;
    text-decoration: none;
}

.result-text {
    clear: left;
    text-align: left;
}

#Q {
    width: 50px;
    height: 50px;
    margin: 2.2em 0 0 1em;
    float: left;
}
</style>
